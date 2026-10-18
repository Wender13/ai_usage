//! Compact window with the current percentages and the alerts of overlay-channel rules.
//! GNOME on Wayland ignores always-on-top and positioning for it unless the app runs
//! through XWayland (see `Settings::use_xwayland`).

use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::rules::Alert;
use crate::settings::{Anchor, OverlayPrefs, Settings};
use crate::state::AppState;
use crate::store;

pub const LABEL: &str = "overlay";
pub const ALERTS_CHANGED: &str = "overlay://alerts";

/// Creates the overlay hidden. Desktop only: mobile apps have a single window.
#[cfg(desktop)]
pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    // Initial size only; the page resizes the window to fit its content.
    const WIDTH: f64 = 220.0;
    const HEIGHT: f64 = 90.0;

    app.manage(placement::Placement::default());
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay.html".into()))
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
    placement::place(app);
    Ok(())
}

pub fn alerts<R: Runtime>(app: &AppHandle<R>) -> Vec<Alert> {
    app.state::<AppState>().overlay_alerts().clone()
}

/// Persists whether the overlay is shown. Turning it off also drops pending alerts.
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<Settings, String> {
    let settings = store::change_settings(app, |settings| settings.overlay_enabled = enabled)?;
    if !enabled {
        app.state::<AppState>().overlay_alerts().clear();
        emit_alerts(app);
    }
    sync_visibility(app);
    #[cfg(desktop)]
    crate::tray::set_overlay_checked(app, enabled);
    Ok(settings)
}

/// Replaces the display preferences, keeping the anchor (see `set_anchor`).
pub fn update_prefs<R: Runtime>(
    app: &AppHandle<R>,
    prefs: OverlayPrefs,
) -> Result<Settings, String> {
    prefs.validate()?;
    store::change_settings(app, |settings| {
        settings.overlay = OverlayPrefs {
            anchor: settings.overlay.anchor,
            ..prefs
        };
    })
}

/// Moves the overlay to a preset position, discarding the position saved by dragging.
pub fn set_anchor<R: Runtime>(app: &AppHandle<R>, anchor: Anchor) -> Result<Settings, String> {
    let settings = store::change_settings(app, |settings| {
        settings.overlay.anchor = anchor;
        settings.overlay_position = None;
    })?;
    #[cfg(desktop)]
    placement::place(app);
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
            // Some window managers drop these while the window is hidden.
            let _ = window.set_always_on_top(true);
            placement::place(app);
        } else {
            let _ = window.hide();
        }
    }
    #[cfg(mobile)]
    let _ = app;
}

#[cfg(desktop)]
pub use placement::{on_moved, on_resized};

/// Where the overlay goes on screen. Positions are physical pixels.
#[cfg(desktop)]
mod placement {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

    use super::LABEL;
    use crate::settings::{Anchor, OverlayPosition};
    use crate::state::{AppState, lock};
    use crate::store;

    /// Distance from the edges of the work area, in logical pixels.
    const MARGIN: f64 = 16.0;
    /// Moves arrive continuously while dragging; only the final position is saved.
    const SAVE_DELAY: Duration = Duration::from_millis(500);
    /// Window managers may report a position a few pixels off the requested one.
    const OWN_MOVE_TOLERANCE: i32 = 2;

    #[derive(Default)]
    pub struct Placement {
        /// Last position set by the app, to tell its own moves from the user's.
        placed: Mutex<Option<PhysicalPosition<i32>>>,
        /// Bumped on every user move; a pending save runs only if no newer move arrived.
        moves: AtomicU64,
        /// Last size seen, since moving the window also reports a resize on GTK.
        size: Mutex<Option<PhysicalSize<u32>>>,
    }

    /// Puts the overlay at its saved position, or at its anchor when there is none.
    pub fn place<R: Runtime>(app: &AppHandle<R>) {
        let Some(window) = app.get_webview_window(LABEL) else {
            return;
        };
        let (anchor, saved) = {
            let state = app.state::<AppState>();
            let settings = state.settings();
            (settings.overlay.anchor, settings.overlay_position)
        };
        let target = match saved {
            Some(position) => Some(PhysicalPosition::new(position.x, position.y)),
            None => anchored_position(&window, anchor),
        };
        let Some(target) = target else {
            return;
        };

        if let Some(placement) = app.try_state::<Placement>() {
            *lock(&placement.placed) = Some(target);
        }
        let _ = window.set_position(target);
    }

    /// Saves where the user dragged the overlay, unless the position is locked.
    pub fn on_moved<R: Runtime>(app: &AppHandle<R>, position: PhysicalPosition<i32>) {
        let Some(placement) = app.try_state::<Placement>() else {
            return;
        };
        let own_move = lock(&placement.placed).is_some_and(|placed| {
            (placed.x - position.x).abs() <= OWN_MOVE_TOLERANCE
                && (placed.y - position.y).abs() <= OWN_MOVE_TOLERANCE
        });
        // A locked overlay returns to its saved position the next time it is placed.
        if own_move || app.state::<AppState>().settings().overlay.locked {
            return;
        }

        let generation = placement.moves.fetch_add(1, Ordering::SeqCst) + 1;
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(SAVE_DELAY).await;
            let Some(placement) = app.try_state::<Placement>() else {
                return;
            };
            if placement.moves.load(Ordering::SeqCst) != generation {
                return;
            }
            *lock(&placement.placed) = Some(position);
            let saved = OverlayPosition {
                x: position.x,
                y: position.y,
            };
            if let Err(error) =
                store::change_settings(&app, |settings| settings.overlay_position = Some(saved))
            {
                eprintln!("{error}");
            }
        });
    }

    /// Keeps an anchored overlay in its corner when its content changes size, so a
    /// bottom anchor grows upwards instead of off screen.
    pub fn on_resized<R: Runtime>(app: &AppHandle<R>, size: PhysicalSize<u32>) {
        let Some(placement) = app.try_state::<Placement>() else {
            return;
        };
        // GTK reports a resize for every move too; placing again on those would loop
        // forever (place, move, resize, place...) and starve the event loop.
        if lock(&placement.size).replace(size) == Some(size) {
            return;
        }
        let anchored = app
            .state::<AppState>()
            .settings()
            .overlay_position
            .is_none();
        if anchored {
            place(app);
        }
    }

    fn anchored_position<R: Runtime>(
        window: &WebviewWindow<R>,
        anchor: Anchor,
    ) -> Option<PhysicalPosition<i32>> {
        let monitor = window
            .current_monitor()
            .ok()
            .flatten()
            .or_else(|| window.primary_monitor().ok().flatten())?;
        let size = window.outer_size().ok()?;
        let area = monitor.work_area();
        let margin = (MARGIN * monitor.scale_factor()).round() as i32;

        let (x, y) = anchor_point(
            anchor,
            (area.position.x, area.position.y),
            (area.size.width as i32, area.size.height as i32),
            (size.width as i32, size.height as i32),
            margin,
        );
        Some(PhysicalPosition::new(x, y))
    }

    /// Top-left corner of a window of `window` size placed at `anchor` inside the area.
    pub(super) fn anchor_point(
        anchor: Anchor,
        (area_x, area_y): (i32, i32),
        (area_width, area_height): (i32, i32),
        (width, height): (i32, i32),
        margin: i32,
    ) -> (i32, i32) {
        let x = match anchor {
            Anchor::TopLeft | Anchor::BottomLeft => area_x + margin,
            Anchor::TopCenter | Anchor::BottomCenter => area_x + (area_width - width) / 2,
            Anchor::TopRight | Anchor::BottomRight => area_x + area_width - width - margin,
        };
        let y = match anchor {
            Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => area_y + margin,
            Anchor::BottomLeft | Anchor::BottomCenter | Anchor::BottomRight => {
                area_y + area_height - height - margin
            }
        };
        (x, y)
    }
}

#[cfg(all(test, desktop))]
mod tests {
    use super::placement::anchor_point;
    use crate::settings::Anchor;

    /// Work area of a 1920x1080 monitor below a 32 px top bar.
    const ORIGIN: (i32, i32) = (0, 32);
    const AREA: (i32, i32) = (1920, 1048);
    const WINDOW: (i32, i32) = (200, 40);
    const MARGIN: i32 = 16;

    fn point(anchor: Anchor) -> (i32, i32) {
        anchor_point(anchor, ORIGIN, AREA, WINDOW, MARGIN)
    }

    #[test]
    fn anchors_respect_work_area_and_margin() {
        assert_eq!(point(Anchor::TopLeft), (16, 48));
        assert_eq!(point(Anchor::TopCenter), (860, 48));
        assert_eq!(point(Anchor::TopRight), (1704, 48));
        assert_eq!(point(Anchor::BottomLeft), (16, 1024));
        assert_eq!(point(Anchor::BottomCenter), (860, 1024));
        assert_eq!(point(Anchor::BottomRight), (1704, 1024));
    }

    #[test]
    fn anchors_follow_the_monitor_origin() {
        // Second monitor to the right of the first one.
        let (x, y) = anchor_point(Anchor::TopRight, (1920, 0), AREA, WINDOW, MARGIN);

        assert_eq!((x, y), (3624, 16));
    }
}
