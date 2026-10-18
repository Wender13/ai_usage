use tauri::{AppHandle, Runtime, State};
use uuid::Uuid;

use crate::domain::UsageSnapshot;
use crate::rules::{Rule, RuleInput};
use crate::settings::Settings;
use crate::state::AppState;
use crate::store;

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

#[tauri::command]
pub fn update_settings<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    settings.validate()?;
    store::save_settings(&app, &settings)?;
    *state.settings() = settings.clone();
    // Restarts the wait so the new interval applies now (this also polls once).
    state.poller.refresh_now();
    Ok(settings)
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
