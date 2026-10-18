//! Persistence of settings, rules and rule state through the store plugin.
//! The file lives in the app data directory resolved by the plugin for each platform.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use crate::rules::{FiredCycles, Rule};
use crate::settings::Settings;

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

pub fn save_settings<R: Runtime>(app: &AppHandle<R>, settings: &Settings) -> Result<(), String> {
    save(app, key::SETTINGS, settings)
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
