//! Tray menu construction.

use tauri::{
    App, Runtime,
    menu::{Menu, MenuItemBuilder, PredefinedMenuItem},
};

pub fn create_tray_menu<R: Runtime>(app: &App<R>) -> Result<Menu<R>, tauri::Error> {
    let chat = MenuItemBuilder::with_id("chat", "Open Chat").build(app)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let status = MenuItemBuilder::with_id("status", "Status: Checking...")
        .enabled(false)
        .build(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit ZeroClaw").build(app)?;

    Menu::with_items(app, &[&chat, &sep1, &status, &sep2, &quit])
}
