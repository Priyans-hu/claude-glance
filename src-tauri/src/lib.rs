pub mod parser;
pub mod process;
pub mod scanner;
pub mod state;
pub mod watcher;

use tauri::{Emitter, Manager};

use crate::state::Session;
use crate::watcher::{refresh, spawn, SessionMap};

const SESSIONS_CHANGED_EVENT: &str = "sessions_changed";

#[tauri::command]
fn list_sessions(state: tauri::State<'_, SessionMap>) -> Vec<Session> {
    let guard = state.lock().unwrap();
    let mut sessions: Vec<Session> = guard.values().cloned().collect();
    sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
    sessions
}

#[tauri::command]
async fn rescan_sessions(
    app: tauri::AppHandle,
    state: tauri::State<'_, SessionMap>,
) -> Result<usize, String> {
    let root = scanner::default_projects_root().ok_or_else(|| "no home directory".to_string())?;
    refresh(&root, &state).map_err(|e| e.to_string())?;

    let snapshot: Vec<Session> = {
        let guard = state.lock().unwrap();
        guard.values().cloned().collect()
    };
    let count = snapshot.len();

    if let Err(err) = app.emit(SESSIONS_CHANGED_EVENT, snapshot) {
        eprintln!("claude-glance: failed to emit rescan update: {err}");
    }
    Ok(count)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = scanner::default_projects_root().unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                    .join(".claude")
                    .join("projects")
            });
            let (map, mut rx) = match spawn(root) {
                Ok(pair) => pair,
                Err(err) => {
                    eprintln!("claude-glance: failed to start watcher: {err}");
                    return Ok(());
                }
            };

            app.manage(map);

            // Relay watcher updates to the frontend via Tauri events. We use
            // the runtime created by Tauri so spawn_blocking is unnecessary.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Some(update) = rx.recv().await {
                    if let Err(err) =
                        app_handle.emit(SESSIONS_CHANGED_EVENT, update.sessions.clone())
                    {
                        eprintln!("claude-glance: failed to emit update: {err}");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_sessions, rescan_sessions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
