mod commands;

use commands::{config, daemon, tasks, window};
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            app.global_shortcut().on_shortcut(
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyD),
                |app, _shortcut, _event| {
                    if let Some(window) = app.get_webview_window("ninja-bar") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                },
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config,
            window::resize_window,
            window::position_window,
            daemon::ensure_daemon_running,
            tasks::list_recent_tasks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
