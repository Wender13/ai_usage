use tauri::State;

use crate::domain::UsageSnapshot;
use crate::state::AppState;

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> UsageSnapshot {
    state.snapshot().clone()
}

#[tauri::command]
pub fn refresh_now(state: State<'_, AppState>) {
    state.poller.refresh_now();
}
