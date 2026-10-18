mod commands;
mod domain;
mod error;
mod poller;
mod providers;
mod state;
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
        .setup(|app| {
            // Without a home directory the path is empty and the provider reports missing credentials.
            let home = app.path().home_dir().unwrap_or_default();
            let providers: Vec<Arc<dyn UsageProvider>> = vec![Arc::new(ClaudeOAuthProvider::new(
                ClaudeOAuthProvider::credentials_path(&home),
            )?)];

            let poller = PollerHandle::default();
            let snapshot = UsageSnapshot::new(providers.iter().map(|p| p.id()));
            app.manage(AppState::new(snapshot, poller.clone()));
            poller::spawn(app.handle().clone(), providers, poller);

            #[cfg(desktop)]
            {
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
            commands::refresh_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
