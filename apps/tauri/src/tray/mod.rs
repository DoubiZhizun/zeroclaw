//! System tray integration for ZeroClaw Desktop.

pub mod events;
pub mod icon;
pub mod menu;

use tauri::{
    App, Manager, Runtime,
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
};

/// Set up the system tray icon and menu.
pub fn setup_tray<R: Runtime>(app: &App<R>) -> Result<TrayIcon<R>, tauri::Error> {
    let menu = menu::create_tray_menu(app)?;

    TrayIconBuilder::with_id("main")
        .tooltip("ZeroClaw — Disconnected")
        .icon(icon::icon_for_state(false, crate::state::AgentStatus::Idle))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(events::handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                position,
                ..
            } = event
                && button == tauri::tray::MouseButton::Left
                && button_state == tauri::tray::MouseButtonState::Up
            {
                let app = tray.app_handle();
                toggle_chat_panel(app, position);
            }
        })
        .build(app)
}

/// Toggle the chat panel near the tray icon click position.
fn toggle_chat_panel<R: Runtime>(
    app: &tauri::AppHandle<R>,
    tray_position: tauri::PhysicalPosition<f64>,
) {
    let Some(chat) = app.get_webview_window("chat") else {
        return;
    };

    if chat.is_visible().unwrap_or(false) {
        let _ = chat.hide();
        return;
    }

    let panel_w = 380.0_f64;

    let x = (tray_position.x - panel_w / 2.0).max(0.0);
    let y = tray_position.y + 4.0;

    let _ = chat.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: x as i32,
        y: y as i32,
    }));
    let _ = chat.show();
    let _ = chat.set_focus();
}
