use tauri::{AppHandle, Runtime, State};
use uuid::Uuid;

use crate::domain::UsageSnapshot;
use crate::rules::{Alert, Rule, RuleInput};
use crate::settings::{Anchor, OverlayPrefs, Settings, validate_poll_interval};
use crate::state::AppState;
use crate::{overlay, store};

const RULE_NOT_FOUND: &str = "Regra não encontrada.";

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> UsageSnapshot {
    state.snapshot().clone()
}

#[tauri::command]
pub fn refresh_now(state: State<'_, AppState>) {
    state.poller.refresh_now();
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings().clone()
}

/// Only the poll interval is taken from `settings`; the other fields have their own
/// commands, so a window holding stale settings cannot revert changes made elsewhere.
#[tauri::command]
pub fn update_settings<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let interval = settings.poll_interval_minutes;
    validate_poll_interval(interval)?;
    let saved = store::change_settings(&app, |current| current.poll_interval_minutes = interval)?;
    // Restarts the wait so the new interval applies now (this also polls once).
    state.poller.refresh_now();
    Ok(saved)
}

#[tauri::command]
pub fn set_overlay_visible<R: Runtime>(
    app: AppHandle<R>,
    visible: bool,
) -> Result<Settings, String> {
    overlay::set_enabled(&app, visible)
}

/// Changes what the overlay shows and how it looks; the anchor and the saved
/// position are kept, since they change through `set_overlay_anchor` and dragging.
#[tauri::command]
pub fn update_overlay_prefs<R: Runtime>(
    app: AppHandle<R>,
    prefs: OverlayPrefs,
) -> Result<Settings, String> {
    overlay::update_prefs(&app, prefs)
}

#[tauri::command]
pub fn set_overlay_anchor<R: Runtime>(
    app: AppHandle<R>,
    anchor: Anchor,
) -> Result<Settings, String> {
    overlay::set_anchor(&app, anchor)
}

#[tauri::command]
pub fn get_overlay_alerts<R: Runtime>(app: AppHandle<R>) -> Vec<Alert> {
    overlay::alerts(&app)
}

#[tauri::command]
pub fn dismiss_overlay_alert<R: Runtime>(app: AppHandle<R>) {
    overlay::dismiss_alert(&app);
}

#[tauri::command]
pub fn list_rules(state: State<'_, AppState>) -> Vec<Rule> {
    state.rules().rules.clone()
}

// Rule changes are saved before being applied in memory, so a failed save leaves
// memory and disk in agreement.

#[tauri::command]
pub fn create_rule<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    input: RuleInput,
) -> Result<Rule, String> {
    input.kind.validate()?;
    let rule = Rule {
        id: Uuid::new_v4().to_string(),
        enabled: input.enabled,
        channel: input.channel,
        kind: input.kind,
    };

    let mut book = state.rules();
    let mut rules = book.rules.clone();
    rules.push(rule.clone());
    store::save_rules(&app, &rules)?;
    book.rules = rules;
    Ok(rule)
}

/// An edited rule starts fresh: it may fire again in the current cycle.
#[tauri::command]
pub fn update_rule<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    rule: Rule,
) -> Result<Rule, String> {
    rule.kind.validate()?;

    let mut book = state.rules();
    let mut rules = book.rules.clone();
    let slot = rules
        .iter_mut()
        .find(|existing| existing.id == rule.id)
        .ok_or(RULE_NOT_FOUND)?;
    *slot = rule.clone();
    store::save_rules(&app, &rules)?;
    book.rules = rules;

    book.state.forget(&rule.id);
    store::save_fired(&app, &book.state.fired)?;
    Ok(rule)
}

#[tauri::command]
pub fn delete_rule<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut book = state.rules();
    if !book.rules.iter().any(|rule| rule.id == id) {
        return Err(RULE_NOT_FOUND.into());
    }
    let rules: Vec<Rule> = book
        .rules
        .iter()
        .filter(|rule| rule.id != id)
        .cloned()
        .collect();
    store::save_rules(&app, &rules)?;
    book.rules = rules;

    book.state.forget(&id);
    store::save_fired(&app, &book.state.fired)
}
