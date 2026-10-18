use tauri::{
    AppHandle, Manager, Runtime,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::state::AppState;
use crate::{overlay, windows};

const TRAY_ID: &str = "main";

mod item {
    pub const OPEN: &str = "open";
    pub const TOGGLE_OVERLAY: &str = "toggle_overlay";
    pub const REFRESH: &str = "refresh";
    pub const QUIT: &str = "quit";
}

/// Kept in managed state so the overlay item can follow changes made from the UI.
struct TrayMenu<R: Runtime> {
    overlay: CheckMenuItem<R>,
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let overlay_enabled = app.state::<AppState>().settings().overlay_enabled;

    let open = MenuItem::with_id(app, item::OPEN, "Abrir janela", true, None::<&str>)?;
    let overlay = CheckMenuItem::with_id(
        app,
        item::TOGGLE_OVERLAY,
        "Overlay",
        true,
        overlay_enabled,
        None::<&str>,
    )?;
    let refresh = MenuItem::with_id(app, item::REFRESH, "Atualizar agora", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, item::QUIT, "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &overlay, &refresh, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("AI Usage")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            item::OPEN => windows::show_main(app),
            item::TOGGLE_OVERLAY => toggle_overlay(app),
            item::REFRESH => app.state::<AppState>().poller.refresh_now(),
            item::QUIT => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    app.manage(TrayMenu { overlay });
    Ok(())
}

pub fn set_overlay_checked<R: Runtime>(app: &AppHandle<R>, checked: bool) {
    if let Some(menu) = app.try_state::<TrayMenu<R>>() {
        let _ = menu.overlay.set_checked(checked);
    }
}

fn toggle_overlay<R: Runtime>(app: &AppHandle<R>) {
    let enabled = !app.state::<AppState>().settings().overlay_enabled;
    if let Err(error) = overlay::set_enabled(app, enabled) {
        eprintln!("{error}");
        // The menu toggled its check mark on click; restore it.
        set_overlay_checked(app, !enabled);
    }
}
