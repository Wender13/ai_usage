mod alerts;
mod commands;
mod domain;
mod error;
mod overlay;
mod poller;
mod providers;
mod rules;
mod settings;
mod state;
mod store;
#[cfg(desktop)]
mod tray;
#[cfg(desktop)]
mod windows;

use std::sync::Arc;

use tauri::Manager;

use domain::UsageSnapshot;
use poller::PollerHandle;
use providers::UsageProvider;
use providers::claude::ClaudeOAuthProvider;
use rules::{RuleBook, RuleState};
use state::AppState;

/// Command-line flag used to start the app with only the tray icon (no main window).
#[cfg(desktop)]
const HIDDEN_FLAG: &str = "--hidden";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // Must be the first plugin registered: a second launch only focuses the running instance.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        windows::show_main(app);
    }));

    builder
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Without a home directory the path is empty and the provider reports missing credentials.
            let home = app.path().home_dir().unwrap_or_default();
            let providers: Vec<Arc<dyn UsageProvider>> = vec![Arc::new(ClaudeOAuthProvider::new(
                ClaudeOAuthProvider::credentials_path(&home),
            )?)];

            let persisted = store::load(app.handle());
            let rule_book = RuleBook {
                rules: persisted.rules,
                state: RuleState::with_fired(persisted.fired),
            };
            let snapshot = UsageSnapshot::new(providers.iter().map(|p| p.id()));
            let poller = PollerHandle::default();
            app.manage(AppState::new(
                snapshot,
                persisted.settings,
                rule_book,
                poller.clone(),
            ));
            poller::spawn(app.handle().clone(), providers, poller);

            #[cfg(desktop)]
            {
                overlay::create(app.handle())?;
                overlay::sync_visibility(app.handle());
                tray::create(app.handle())?;

                if !std::env::args().any(|arg| arg == HIDDEN_FLAG) {
                    windows::show_main(app.handle());
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            windows::handle_event(window, event);
            #[cfg(mobile)]
            let _ = (window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::refresh_now,
            commands::get_settings,
            commands::update_settings,
            commands::list_rules,
            commands::create_rule,
            commands::update_rule,
            commands::delete_rule,
            commands::set_overlay_visible,
            commands::update_overlay_prefs,
            commands::set_overlay_anchor,
            commands::get_overlay_alerts,
            commands::dismiss_overlay_alert,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
