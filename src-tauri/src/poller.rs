use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::Notify;

use crate::providers::UsageProvider;
use crate::state::AppState;

pub const USAGE_UPDATED: &str = "usage://updated";

const DEFAULT_INTERVAL: Duration = Duration::from_secs(3 * 60);

/// Wakes the poller ahead of its schedule. Requests made while a poll is
/// running are coalesced into a single extra poll right after it.
#[derive(Clone, Default)]
pub struct PollerHandle {
    wake: Arc<Notify>,
}

impl PollerHandle {
    pub fn refresh_now(&self) {
        self.wake.notify_one();
    }
}

/// Runs the polling loop on the Tauri async runtime, independent of any window.
pub fn spawn<R: Runtime>(
    app: AppHandle<R>,
    providers: Vec<Arc<dyn UsageProvider>>,
    handle: PollerHandle,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            poll_once(&app, &providers).await;
            tokio::select! {
                _ = tokio::time::sleep(DEFAULT_INTERVAL) => {}
                _ = handle.wake.notified() => {}
            }
        }
    });
}

async fn poll_once<R: Runtime>(app: &AppHandle<R>, providers: &[Arc<dyn UsageProvider>]) {
    let mut results = Vec::with_capacity(providers.len());
    for provider in providers {
        results.push((provider.id(), provider.fetch().await));
    }

    let now = Utc::now();
    let state = app.state::<AppState>();
    let snapshot = {
        let mut snapshot = state.snapshot();
        for (provider, result) in results {
            snapshot.record(provider, result, now);
        }
        snapshot.fetched_at = Some(now);
        snapshot.clone()
    };

    let _ = app.emit(USAGE_UPDATED, &snapshot);
}
