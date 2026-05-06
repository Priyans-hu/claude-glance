pub mod cleanup;
pub mod parser;
pub mod process;
pub mod scanner;
pub mod state;
pub mod watcher;

use std::path::PathBuf;

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

/// Resolve the JSONL transcript path for `session_id` from the in-memory
/// session map. Returns an error if the session id is unknown so callers
/// surface a helpful message instead of an opaque IPC failure.
fn transcript_path_for(state: &SessionMap, session_id: &str) -> Result<PathBuf, String> {
    let guard = state.lock().unwrap();
    guard
        .get(session_id)
        .map(|s| s.transcript_path.clone())
        .ok_or_else(|| format!("unknown session id: {session_id}"))
}

#[tauri::command]
async fn stop_session(
    session_id: String,
    state: tauri::State<'_, SessionMap>,
) -> Result<bool, String> {
    let path = transcript_path_for(&state, &session_id)?;
    let Some(pid) = process::find_holder_pid(&path) else {
        return Ok(false);
    };
    process::stop_pid(pid).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
async fn delete_session(
    session_id: String,
    state: tauri::State<'_, SessionMap>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let path = transcript_path_for(&state, &session_id)?;

    // Stop any live holder first so it can't recreate the file mid-delete.
    if let Some(pid) = process::find_holder_pid(&path) {
        process::stop_pid(pid).map_err(|e| e.to_string())?;
    }

    cleanup::delete_session_files(&path).map_err(|e| e.to_string())?;

    let snapshot: Vec<Session> = {
        let mut guard = state.lock().unwrap();
        guard.remove(&session_id);
        guard.values().cloned().collect()
    };

    if let Err(err) = app.emit(SESSIONS_CHANGED_EVENT, snapshot) {
        eprintln!("claude-glance: failed to emit delete update: {err}");
    }
    Ok(())
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
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            rescan_sessions,
            stop_session,
            delete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
