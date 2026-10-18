use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

use crate::overlay;

pub const MAIN: &str = "main";

pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Closing the main window hides it; the app keeps running in the tray until "Sair".
/// Closing the overlay (e.g. Alt+F4) turns it off, like the tray toggle.
pub fn handle_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    match window.label() {
        MAIN => {
            api.prevent_close();
            let _ = window.hide();
        }
        overlay::LABEL => {
            api.prevent_close();
            if let Err(error) = overlay::set_enabled(window.app_handle(), false) {
                eprintln!("{error}");
            }
        }
        _ => {}
    }
}
