use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::UsageSnapshot;
use crate::poller::PollerHandle;
use crate::rules::{Alert, RuleBook};
use crate::settings::Settings;

pub struct AppState {
    snapshot: Mutex<UsageSnapshot>,
    settings: Mutex<Settings>,
    rules: Mutex<RuleBook>,
    /// Alerts waiting to be dismissed in the overlay, oldest first.
    overlay_alerts: Mutex<Vec<Alert>>,
    pub poller: PollerHandle,
}

impl AppState {
    pub fn new(
        snapshot: UsageSnapshot,
        settings: Settings,
        rules: RuleBook,
        poller: PollerHandle,
    ) -> Self {
        Self {
            snapshot: Mutex::new(snapshot),
            settings: Mutex::new(settings),
            rules: Mutex::new(rules),
            overlay_alerts: Mutex::new(Vec::new()),
            poller,
        }
    }

    pub fn snapshot(&self) -> MutexGuard<'_, UsageSnapshot> {
        lock(&self.snapshot)
    }

    pub fn settings(&self) -> MutexGuard<'_, Settings> {
        lock(&self.settings)
    }

    pub fn rules(&self) -> MutexGuard<'_, RuleBook> {
        lock(&self.rules)
    }

    pub fn overlay_alerts(&self) -> MutexGuard<'_, Vec<Alert>> {
        lock(&self.overlay_alerts)
    }
}

/// Never panics: after a panic elsewhere the data is still structurally valid,
/// and keeping the app running beats crashing on the next access.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
