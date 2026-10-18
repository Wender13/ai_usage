#[cfg(desktop)]
mod tray;
#[cfg(desktop)]
mod windows;

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
            #[cfg(desktop)]
            {
                tray::create(app.handle())?;

                if !std::env::args().any(|arg| arg == HIDDEN_FLAG) {
                    windows::show_main(app.handle());
                }
            }
            #[cfg(mobile)]
            let _ = app;

            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            windows::handle_event(window, event);
            #[cfg(mobile)]
            let _ = (window, event);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
