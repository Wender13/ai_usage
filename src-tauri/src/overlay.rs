//! Compact window with the current percentages and the alerts of overlay-channel rules.
//! GNOME on Wayland ignores always-on-top and positioning for it; see the README.

use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::rules::Alert;
use crate::settings::{SETTINGS_CHANGED, Settings};
use crate::state::AppState;
use crate::store;

pub const LABEL: &str = "overlay";
pub const ALERTS_CHANGED: &str = "overlay://alerts";

/// Creates the overlay hidden. Desktop only: mobile apps have a single window.
#[cfg(desktop)]
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    use tauri::{LogicalPosition, WebviewUrl, WebviewWindowBuilder};

    // Initial size only; the page resizes the window to fit its content.
    const WIDTH: f64 = 220.0;
    const HEIGHT: f64 = 90.0;
    // Room for the content to grow rightwards without leaving the screen.
    const RIGHT_OFFSET: f64 = 340.0;
    const TOP_OFFSET: f64 = 48.0;

    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay.html".into()))
        .title("AI Usage Overlay")
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        // GTK ignores programmatic resizes of non-resizable windows, which the page needs
        // to fit the window to its content. Without decorations there is no border to drag.
        .resizable(true)
        // Showing the overlay for an alert must not steal focus from what the user is doing.
        .focusable(false)
        .visible(false)
        .build()?;

    if let Ok(Some(monitor)) = window.primary_monitor() {
        let scale = monitor.scale_factor();
        let origin = monitor.position().to_logical::<f64>(scale);
        let size = monitor.size().to_logical::<f64>(scale);
        let _ = window.set_position(LogicalPosition::new(
            origin.x + size.width - RIGHT_OFFSET,
            origin.y + TOP_OFFSET,
        ));
    }
    Ok(())
}

pub fn alerts<R: Runtime>(app: &AppHandle<R>) -> Vec<Alert> {
    app.state::<AppState>().overlay_alerts().clone()
}

/// Persists whether the overlay is shown. Turning it off also drops pending alerts.
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<Settings, String> {
    let state = app.state::<AppState>();
    let mut settings = state.settings().clone();
    settings.overlay_enabled = enabled;
    store::save_settings(app, &settings)?;
    *state.settings() = settings.clone();

    if !enabled {
        state.overlay_alerts().clear();
        emit_alerts(app);
    }
    sync_visibility(app);
    #[cfg(desktop)]
    crate::tray::set_overlay_checked(app, enabled);
    let _ = app.emit(SETTINGS_CHANGED, &settings);
    Ok(settings)
}

pub fn push_alerts<R: Runtime>(app: &AppHandle<R>, alerts: Vec<Alert>) {
    if alerts.is_empty() {
        return;
    }
    app.state::<AppState>().overlay_alerts().extend(alerts);
    emit_alerts(app);
    sync_visibility(app);
}

/// Removes the oldest alert; the overlay hides again if it was only shown for alerts.
pub fn dismiss_alert<R: Runtime>(app: &AppHandle<R>) {
    {
        let state = app.state::<AppState>();
        let mut queue = state.overlay_alerts();
        if !queue.is_empty() {
            queue.remove(0);
        }
    }
    emit_alerts(app);
    sync_visibility(app);
}

fn emit_alerts<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit_to(LABEL, ALERTS_CHANGED, alerts(app));
}

/// The overlay is visible while enabled, or while it has alerts to show.
pub fn sync_visibility<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(desktop)]
    {
        let state = app.state::<AppState>();
        let enabled = state.settings().overlay_enabled;
        let has_alerts = !state.overlay_alerts().is_empty();
        let Some(window) = app.get_webview_window(LABEL) else {
            return;
        };

        if enabled || has_alerts {
            let _ = window.show();
            // Some window managers drop the flag while the window is hidden.
            let _ = window.set_always_on_top(true);
        } else {
            let _ = window.hide();
        }
    }
    #[cfg(mobile)]
    let _ = app;
}
