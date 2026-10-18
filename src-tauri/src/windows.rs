use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

pub const MAIN: &str = "main";

pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Closing the main window hides it; the app keeps running in the tray until "Sair".
pub fn handle_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event
        && window.label() == MAIN
    {
        api.prevent_close();
        let _ = window.hide();
    }
}
