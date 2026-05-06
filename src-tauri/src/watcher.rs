//! Filesystem watcher that keeps the in-memory session map in sync with
//! `~/.claude/projects/**`.
//!
//! Runs in a dedicated tokio task. On any debounced filesystem event, the
//! affected `.jsonl` is re-loaded (or removed) and a `WatchUpdate` is
//! published to subscribers via a tokio mpsc channel.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use tokio::sync::mpsc;

use crate::scanner::{load_session, scan, scan_at};
use crate::state::Session;

pub type SessionMap = Arc<Mutex<HashMap<String, Session>>>;

/// Update payload emitted on every observed change.
#[derive(Debug, Clone)]
pub struct WatchUpdate {
    pub sessions: Vec<Session>,
}

/// Spawn the watcher task. Returns the shared session map (already
/// populated with the initial scan) and a receiver that fires a
/// `WatchUpdate` whenever the map changes.
pub fn spawn(root: PathBuf) -> Result<(SessionMap, mpsc::Receiver<WatchUpdate>)> {
    let map: SessionMap = Arc::new(Mutex::new(scan(&root).unwrap_or_default()));
    let (tx, rx) = mpsc::channel::<WatchUpdate>(32);

    // Notify uses a sync mpsc internally; we relay events to a tokio mpsc so
    // the main async task can await them without blocking.
    let (raw_tx, raw_rx) = std::sync::mpsc::channel::<notify::Result<Vec<PathBuf>>>();

    let map_for_thread = Arc::clone(&map);
    let root_for_thread = root.clone();
    std::thread::spawn(move || {
        // Second arg is the poll-tick fallback. FSEvents on macOS sometimes
        // coalesces or drops `Create` events for newly-spawned session
        // files; a 5s poll catches those without flooding the channel.
        let mut debouncer = match new_debouncer(
            Duration::from_millis(250),
            Some(Duration::from_secs(5)),
            move |res: notify_debouncer_full::DebounceEventResult| match res {
                Ok(events) => {
                    let paths: Vec<PathBuf> =
                        events.into_iter().flat_map(|e| e.event.paths).collect();
                    let _ = raw_tx.send(Ok(paths));
                }
                Err(errors) => {
                    if let Some(first) = errors.into_iter().next() {
                        let _ = raw_tx.send(Err(first));
                    }
                }
            },
        ) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("watcher: failed to construct debouncer: {err}");
                return;
            }
        };

        if let Err(err) = debouncer
            .watcher()
            .watch(&root_for_thread, RecursiveMode::Recursive)
        {
            eprintln!(
                "watcher: failed to watch {}: {err}",
                root_for_thread.display()
            );
            return;
        }

        // Pump filesystem events from the sync channel and relay updates.
        while let Ok(msg) = raw_rx.recv() {
            match msg {
                Ok(paths) => {
                    apply_paths(&paths, &map_for_thread);
                    let snapshot: Vec<Session> =
                        map_for_thread.lock().unwrap().values().cloned().collect();
                    if tx
                        .blocking_send(WatchUpdate { sessions: snapshot })
                        .is_err()
                    {
                        break; // receiver gone
                    }
                }
                Err(err) => {
                    eprintln!("watcher: notify error: {err}");
                }
            }
        }
    });

    Ok((map, rx))
}

/// Apply a batch of changed filesystem paths to the in-memory session map.
/// Pulled out so the unit tests don't need a real watcher.
pub fn apply_paths(paths: &[PathBuf], map: &SessionMap) {
    apply_paths_at(paths, map, Utc::now())
}

pub fn apply_paths_at(paths: &[PathBuf], map: &SessionMap, now: DateTime<Utc>) {
    for path in paths {
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        let Some(session_id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.exists() {
            if let Some(session) = load_session(path, now) {
                map.lock().unwrap().insert(session.id.clone(), session);
            }
        } else {
            map.lock().unwrap().remove(session_id);
        }
    }
}

/// Bulk re-scan helper used at startup if a refresh is requested.
pub fn refresh(root: &Path, map: &SessionMap) -> Result<()> {
    refresh_at(root, map, Utc::now())
}

pub fn refresh_at(root: &Path, map: &SessionMap, now: DateTime<Utc>) -> Result<()> {
    let fresh = scan_at(root, now).context("scan failed")?;
    let mut guard = map.lock().unwrap();
    *guard = fresh;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    fn fixture_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 30, 8, 0, 0).unwrap()
    }

    #[test]
    fn apply_paths_inserts_new_session() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();
        let path = project.join("aaaa.jsonl");
        let mut f = File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"type":"user","timestamp":"2026-03-30T04:33:28Z","cwd":"/Users/x/proj","message":{{"role":"user","content":"hi"}}}}"#
        )
        .unwrap();

        let map: SessionMap = Arc::new(Mutex::new(HashMap::new()));
        apply_paths_at(&[path], &map, fixture_now());

        let guard = map.lock().unwrap();
        assert_eq!(guard.len(), 1);
        assert!(guard.contains_key("aaaa"));
    }

    #[test]
    fn apply_paths_removes_deleted_session() {
        let map: SessionMap = Arc::new(Mutex::new(HashMap::new()));
        // Pre-seed a session.
        {
            let mut guard = map.lock().unwrap();
            guard.insert(
                "ghost".into(),
                Session {
                    id: "ghost".into(),
                    cwd: String::new(),
                    project: String::new(),
                    branch: None,
                    title: String::new(),
                    status: crate::state::SessionStatus::Idle,
                    current_tool: None,
                    permission_mode: None,
                    subagent_count: 0,
                    last_activity: String::new(),
                    tokens: 0,
                    transcript_path: PathBuf::new(),
                },
            );
        }
        // Path to a non-existent jsonl with stem "ghost".
        let dir = tempdir().unwrap();
        let path = dir.path().join("ghost.jsonl");
        apply_paths_at(&[path], &map, fixture_now());
        assert!(map.lock().unwrap().is_empty());
    }

    #[test]
    fn apply_paths_ignores_non_jsonl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("README.md");
        File::create(&path).unwrap();
        let map: SessionMap = Arc::new(Mutex::new(HashMap::new()));
        apply_paths_at(&[path], &map, fixture_now());
        assert!(map.lock().unwrap().is_empty());
    }

    #[test]
    fn refresh_replaces_map() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();
        let path = project.join("zz.jsonl");
        let mut f = File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"type":"user","timestamp":"2026-03-30T04:33:28Z","cwd":"/Users/x/proj","message":{{"role":"user","content":"go"}}}}"#
        )
        .unwrap();

        let map: SessionMap = Arc::new(Mutex::new(HashMap::new()));
        refresh_at(dir.path(), &map, fixture_now()).unwrap();
        assert_eq!(map.lock().unwrap().len(), 1);
    }
}
