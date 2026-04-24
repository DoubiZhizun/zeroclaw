//! Tray menu event handling.

use tauri::{AppHandle, Manager, Runtime, menu::MenuEvent};

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    match event.id().as_ref() {
        "chat" => {
            if let Some(chat) = app.get_webview_window("chat") {
                let _ = chat.show();
                let _ = chat.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}
