use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::Notify;

use crate::domain::UsageWindow;
use crate::providers::UsageProvider;
use crate::rules::{RuleBook, engine};
use crate::state::AppState;
use crate::{alerts, i18n, store};

pub const USAGE_UPDATED: &str = "usage://updated";

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
            // Read on every cycle so a changed interval applies from the next wait.
            let interval = app.state::<AppState>().settings().poll_interval();
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
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

    // Rules only see data fetched in this poll, never stale windows.
    let fresh: Vec<UsageWindow> = results
        .iter()
        .filter_map(|(_, result)| result.as_ref().ok())
        .flatten()
        .cloned()
        .collect();

    let snapshot = {
        let mut snapshot = state.snapshot();
        for (provider, result) in results {
            snapshot.record(provider, result, now);
        }
        snapshot.fetched_at = Some(now);
        snapshot.clone()
    };
    let _ = app.emit(USAGE_UPDATED, &snapshot);

    let (alerts, fired) = {
        let mut book = state.rules();
        let RuleBook { rules, state } = &mut *book;
        let before = state.fired.clone();
        let alerts = engine::evaluate(rules, &fresh, state, now, i18n::current());
        let changed = (state.fired != before).then(|| state.fired.clone());
        (alerts, changed)
    };

    if let Some(fired) = fired
        && let Err(error) = store::save_fired(app, &fired)
    {
        eprintln!("{error}");
    }
    alerts::dispatch(app, alerts);
}
