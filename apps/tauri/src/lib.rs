//! ZeroClaw Desktop — menu bar chat agent powered by the ZeroClaw gateway.

pub mod commands;
pub mod gateway_client;
pub mod health;
pub mod local_node;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod state;
pub mod tray;

use gateway_client::GatewayClient;
use state::shared_state;
use tauri::{Manager, RunEvent};

/// Attempt to auto-pair with the gateway so the chat panel has a valid token.
async fn auto_pair(state: &state::SharedState) -> Option<String> {
    let url = {
        let s = state.read().await;
        s.gateway_url.clone()
    };

    let client = GatewayClient::new(&url, None);

    if !client.requires_pairing().await.unwrap_or(false) {
        return None;
    }

    {
        let s = state.read().await;
        if let Some(ref token) = s.token {
            let authed = GatewayClient::new(&url, Some(token));
            if authed.validate_token().await.unwrap_or(false) {
                return Some(token.clone());
            }
        }
    }

    let client = GatewayClient::new(&url, None);
    match client.auto_pair().await {
        Ok(token) => {
            let mut s = state.write().await;
            s.token = Some(token.clone());
            Some(token)
        }
        Err(_) => None,
    }
}

/// Inject bearer token into the chat panel's localStorage.
/// Uses Tauri's WebviewWindow scripting API — the standard way to pass data
/// from the Rust backend into the embedded webview.
fn inject_token<R: tauri::Runtime>(app: &tauri::AppHandle<R>, token: &str) {
    let escaped = token.replace('\\', "\\\\").replace('\'', "\\'");
    let script = format!("localStorage.setItem('zeroclaw_token', '{escaped}')");
    if let Some(w) = app.get_webview_window("chat") {
        let _ = w.eval(&script);
    }
}

/// Set the macOS dock icon programmatically for dev builds.
#[cfg(target_os = "macos")]
fn set_dock_icon() {
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::NSApplication;
    use objc2_app_kit::NSImage;
    use objc2_foundation::NSData;

    let icon_bytes = include_bytes!("../icons/128x128.png");
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let data = NSData::with_bytes(icon_bytes);
    if let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) {
        let app = NSApplication::sharedApplication(mtm);
        unsafe { app.setApplicationIconImage(Some(&image)) };
    }
}

/// Configure and run the Tauri menu bar application.
pub fn run() {
    let shared = shared_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("chat") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .manage(shared.clone())
        .invoke_handler(tauri::generate_handler![
            commands::agent::send_message,
            commands::gateway::get_status,
            commands::gateway::get_health,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            set_dock_icon();

            let _ = tray::setup_tray(app);

            let app_handle = app.handle().clone();
            let pair_state = shared.clone();
            tauri::async_runtime::spawn(async move {
                for _ in 0..10 {
                    if let Some(token) = auto_pair(&pair_state).await {
                        inject_token(&app_handle, &token);
                        return;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            });

            health::spawn_health_poller(app.handle().clone(), shared.clone());
            local_node::spawn_local_node(shared.clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
