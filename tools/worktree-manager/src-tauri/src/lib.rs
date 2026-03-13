pub mod claude;
pub mod external;
pub mod hooks_config;
pub mod hooks_server;
pub mod session_manager;
pub mod tray;
pub mod types;
pub mod watcher;
pub mod worktree;

use session_manager::SessionManager;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(SessionManager::new())
        .setup(|app| {
            // Setup tray icon
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("Failed to setup tray: {e}");
            }

            let app_handle = app.handle().clone();

            // Start hooks server and configure settings.json
            tauri::async_runtime::spawn(async move {
                match hooks_server::start_hooks_server().await {
                    Ok((port, sender)) => {
                        eprintln!("Hooks server started on port {port}");

                        // Update Claude settings.json with hook URLs
                        if let Err(e) = hooks_config::update_settings(port) {
                            eprintln!("Failed to update Claude settings: {e}");
                        }

                        // Track previous question count for notifications
                        let mut prev_question_count: usize = 0;

                        // Forward hook events to session manager and emit to frontend
                        let mut rx = sender.subscribe();
                        loop {
                            match rx.recv().await {
                                Ok(event) => {
                                    // Update session manager
                                    let manager = app_handle.state::<SessionManager>();
                                    if let Some(session) =
                                        manager.handle_hook_event(&event)
                                    {
                                        // Emit to frontend
                                        let _ = app_handle.emit("hook-event", &event);
                                        let _ = app_handle.emit("session-update", &session);

                                        // Check if question count increased
                                        let current_count = manager.question_count();
                                        if current_count > prev_question_count {
                                            tray::send_question_notification(
                                                &app_handle,
                                                current_count,
                                            );
                                        }
                                        prev_question_count = current_count;
                                    }
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                    eprintln!("Hook event receiver lagged by {n}");
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                    eprintln!("Hook event channel closed");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to start hooks server: {e}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            worktree::list_worktrees,
            worktree::create_worktree,
            worktree::remove_worktree,
            worktree::get_worktree_status,
            worktree::cleanup_merged,
            session_manager::get_session_status,
            session_manager::get_all_sessions,
            session_manager::get_activity_log,
            claude::launch_claude,
            external::open_in_ide,
            external::open_difftool,
            external::open_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
