use tauri::{
    AppHandle, Runtime,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::windows;

const TRAY_ID: &str = "main";

mod item {
    pub const OPEN: &str = "open";
    pub const TOGGLE_OVERLAY: &str = "toggle_overlay";
    pub const REFRESH: &str = "refresh";
    pub const QUIT: &str = "quit";
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, item::OPEN, "Abrir janela", true, None::<&str>)?;
    // Overlay and refresh stay disabled until their features exist.
    let overlay = CheckMenuItem::with_id(
        app,
        item::TOGGLE_OVERLAY,
        "Overlay",
        false,
        false,
        None::<&str>,
    )?;
    let refresh = MenuItem::with_id(app, item::REFRESH, "Atualizar agora", false, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, item::QUIT, "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &overlay, &refresh, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("AI Usage")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            item::OPEN => windows::show_main(app),
            item::QUIT => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}
