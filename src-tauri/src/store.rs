//! Persistence of settings, rules and rule state through the store plugin.
//! The file lives in the app data directory resolved by the plugin for each platform.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::rules::{FiredCycles, Rule};
use crate::settings::{SETTINGS_CHANGED, Settings};
use crate::state::AppState;

const STORE_FILE: &str = "ai-usage.json";

mod key {
    pub const SETTINGS: &str = "settings";
    pub const RULES: &str = "rules";
    pub const RULE_STATE: &str = "ruleState";
}

#[derive(Default)]
pub struct Persisted {
    pub settings: Settings,
    pub rules: Vec<Rule>,
    pub fired: FiredCycles,
}

/// Loads everything, falling back to defaults for missing or unreadable values
/// so a damaged file never prevents the app from starting.
pub fn load<R: Runtime>(app: &AppHandle<R>) -> Persisted {
    let store = match app.store(STORE_FILE) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("could not open {STORE_FILE}, using defaults: {error}");
            return Persisted::default();
        }
    };

    // Rules are read one by one so a single unreadable entry does not discard the others.
    let rules = match store.get(key::RULES) {
        Some(Value::Array(items)) => items
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect(),
        _ => Vec::new(),
    };

    Persisted {
        settings: read(store.get(key::SETTINGS)),
        rules,
        fired: read(store.get(key::RULE_STATE)),
    }
}

/// Applies a change to the settings: saves it, then updates memory and notifies
/// every window. Memory is left untouched if the save fails.
pub fn change_settings<R: Runtime>(
    app: &AppHandle<R>,
    change: impl FnOnce(&mut Settings),
) -> Result<Settings, String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings().clone();
    change(&mut settings);
    save(app, key::SETTINGS, &settings)?;
    *state.settings() = settings.clone();
    let _ = app.emit(SETTINGS_CHANGED, &settings);
    Ok(settings)
}

/// Reads `useXwayland` straight from the store file, since it must be known before
/// Tauri (and GTK) starts. The path mirrors Tauri's app data directory on Linux.
#[cfg(target_os = "linux")]
pub fn xwayland_preference(identifier: &str) -> bool {
    use std::path::PathBuf;

    let data_dir = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    let Some(path) = data_dir.map(|dir| dir.join(identifier).join(STORE_FILE)) else {
        return false;
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|root| root.get(key::SETTINGS)?.get("useXwayland")?.as_bool())
        .unwrap_or(false)
}

pub fn save_rules<R: Runtime>(app: &AppHandle<R>, rules: &[Rule]) -> Result<(), String> {
    save(app, key::RULES, rules)
}

pub fn save_fired<R: Runtime>(app: &AppHandle<R>, fired: &FiredCycles) -> Result<(), String> {
    save(app, key::RULE_STATE, fired)
}

fn read<T: DeserializeOwned + Default>(value: Option<Value>) -> T {
    value
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

fn save<R: Runtime, T: Serialize + ?Sized>(
    app: &AppHandle<R>,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let failed =
        |error: &dyn std::fmt::Display| format!("Não foi possível salvar ({key}): {error}");

    let value = serde_json::to_value(value).map_err(|error| failed(&error))?;
    let store = app.store(STORE_FILE).map_err(|error| failed(&error))?;
    store.set(key, value);
    store.save().map_err(|error| failed(&error))
}
