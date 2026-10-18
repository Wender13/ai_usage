use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::domain::UsageSnapshot;
use crate::poller::PollerHandle;

pub struct AppState {
    snapshot: Mutex<UsageSnapshot>,
    pub poller: PollerHandle,
}

impl AppState {
    pub fn new(snapshot: UsageSnapshot, poller: PollerHandle) -> Self {
        Self {
            snapshot: Mutex::new(snapshot),
            poller,
        }
    }

    /// Never panics: a poisoned lock still holds a consistent snapshot,
    /// since every update replaces fields without intermediate states.
    pub fn snapshot(&self) -> MutexGuard<'_, UsageSnapshot> {
        self.snapshot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
