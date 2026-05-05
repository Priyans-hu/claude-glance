//! Filesystem scanner: walk `~/.claude/projects/**/*.jsonl`, parse every
//! line, and assemble the `HashMap<sessionId, Session>` the UI consumes.
//!
//! Designed to be cheap to call repeatedly so the watcher can use it for
//! both the initial load and per-file refreshes.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};

use crate::parser::{parse_line, SessionEvent};
use crate::state::{build_session, Session};

/// Sessions older than this are filtered out at the scanner level so the
/// frontend never has to think about them. Matches RECENT_AGE_LIMIT in
/// `state.rs` semantically, but kept local to keep the scanner standalone.
const STALE_AGE_DAYS: i64 = 7;

/// Resolve the default Claude Code projects root: `~/.claude/projects`.
pub fn default_projects_root() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

/// Scan a projects root and return all sessions, keyed by session id.
/// Convenience wrapper that uses wall-clock `now`.
pub fn scan(root: &Path) -> Result<HashMap<String, Session>> {
    scan_at(root, Utc::now())
}

/// Same as `scan`, with `now` injected for tests.
pub fn scan_at(root: &Path, now: DateTime<Utc>) -> Result<HashMap<String, Session>> {
    let mut sessions = HashMap::new();
    if !root.exists() {
        return Ok(sessions);
    }

    for project_entry in
        fs::read_dir(root).with_context(|| format!("read_dir {}", root.display()))?
    {
        let Ok(project_entry) = project_entry else {
            continue;
        };
        let project_dir = project_entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        let Ok(read_dir) = fs::read_dir(&project_dir) else {
            continue;
        };
        for file_entry in read_dir.flatten() {
            let path = file_entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            if let Some(session) = load_session(&path, now) {
                sessions.insert(session.id.clone(), session);
            }
        }
    }
    Ok(sessions)
}

/// Read a single `.jsonl` file and turn it into a `Session`. Returns `None`
/// if the file is empty, unreadable, or older than `STALE_AGE_DAYS`.
pub fn load_session(path: &Path, now: DateTime<Utc>) -> Option<Session> {
    let session_id = path.file_stem()?.to_str()?.to_string();
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut events: Vec<SessionEvent> = Vec::new();
    for line in reader.lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(ev) = parse_line(line) {
            events.push(ev);
        }
    }
    if events.is_empty() {
        return None;
    }
    // Sort ascending by timestamp; events without timestamps stay in their
    // original (file) order.
    events.sort_by_key(|e| e.timestamp);

    // Drop stale sessions before doing any extra work.
    let latest = events.iter().rev().find_map(|e| e.timestamp)?;
    if (now - latest) > Duration::days(STALE_AGE_DAYS) {
        return None;
    }

    let subagent_count = count_subagents(path, &session_id);
    Some(build_session(session_id, &events, subagent_count, now))
}

fn count_subagents(jsonl_path: &Path, session_id: &str) -> u32 {
    let parent = match jsonl_path.parent() {
        Some(p) => p,
        None => return 0,
    };
    let subagent_dir = parent.join(session_id).join("subagents");
    let Ok(read_dir) = fs::read_dir(&subagent_dir) else {
        return 0;
    };
    read_dir
        .flatten()
        .filter(|e| {
            let p = e.path();
            p.extension().and_then(|s| s.to_str()) == Some("jsonl")
                && p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("agent-"))
                    .unwrap_or(false)
        })
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_jsonl(path: &Path, lines: &[&str]) {
        let mut f = File::create(path).unwrap();
        for l in lines {
            writeln!(f, "{l}").unwrap();
        }
    }

    #[test]
    fn scan_empty_root_returns_empty() {
        let dir = tempdir().unwrap();
        let sessions = scan(dir.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn scan_missing_root_returns_empty() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        let sessions = scan(&missing).unwrap();
        assert!(sessions.is_empty());
    }

    fn fixture_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 30, 8, 0, 0).unwrap()
    }

    #[test]
    fn scan_picks_up_jsonl_files() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();
        let session_id = "111e1111-e29b-41d4-a716-446655440000";
        let path = project.join(format!("{session_id}.jsonl"));
        write_jsonl(
            &path,
            &[
                r#"{"type":"user","timestamp":"2026-03-30T04:33:28Z","sessionId":"111e1111-e29b-41d4-a716-446655440000","cwd":"/Users/x/proj","gitBranch":"main","permissionMode":"default","message":{"role":"user","content":"hello"}}"#,
                r#"{"type":"assistant","timestamp":"2026-03-30T04:33:30Z","message":{"role":"assistant","content":[{"type":"text","text":"hi"}],"usage":{"input_tokens":5,"output_tokens":2}}}"#,
            ],
        );

        let sessions = scan_at(dir.path(), fixture_now()).unwrap();
        assert_eq!(sessions.len(), 1);
        let s = sessions.get(session_id).unwrap();
        assert_eq!(s.project, "proj");
        assert_eq!(s.title, "hello");
        assert_eq!(s.branch.as_deref(), Some("main"));
        assert_eq!(s.tokens, 7);
    }

    #[test]
    fn scan_counts_subagents() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();
        let session_id = "abc";
        let jsonl = project.join("abc.jsonl");
        write_jsonl(
            &jsonl,
            &[
                r#"{"type":"user","timestamp":"2026-03-30T04:33:28Z","cwd":"/Users/x/proj","message":{"role":"user","content":"go"}}"#,
            ],
        );
        let subagents = project.join("abc").join("subagents");
        fs::create_dir_all(&subagents).unwrap();
        File::create(subagents.join("agent-1.jsonl")).unwrap();
        File::create(subagents.join("agent-2.jsonl")).unwrap();
        // Non-jsonl & non-prefixed should be ignored.
        File::create(subagents.join("notes.txt")).unwrap();

        let sessions = scan_at(dir.path(), fixture_now()).unwrap();
        assert_eq!(sessions.get(session_id).unwrap().subagent_count, 2);
    }

    #[test]
    fn scan_skips_non_jsonl_files() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-y");
        fs::create_dir_all(&project).unwrap();
        File::create(project.join("README.md")).unwrap();
        File::create(project.join("sessions-index.json")).unwrap();

        let sessions = scan(dir.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn load_session_skips_files_older_than_seven_days() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();

        // "now" anchored at 2026-04-10. The fresh session is 1 day old; the
        // stale one is 8 days old.
        let now = Utc.with_ymd_and_hms(2026, 4, 10, 12, 0, 0).unwrap();

        let fresh_path = project.join("fresh.jsonl");
        write_jsonl(
            &fresh_path,
            &[
                r#"{"type":"user","timestamp":"2026-04-09T12:00:00Z","cwd":"/Users/x/proj","message":{"role":"user","content":"hi"}}"#,
            ],
        );
        let stale_path = project.join("stale.jsonl");
        write_jsonl(
            &stale_path,
            &[
                r#"{"type":"user","timestamp":"2026-04-02T12:00:00Z","cwd":"/Users/x/proj","message":{"role":"user","content":"old"}}"#,
            ],
        );

        assert!(load_session(&fresh_path, now).is_some());
        assert!(load_session(&stale_path, now).is_none());
    }

    #[test]
    fn scan_handles_blank_lines_and_invalid_json() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();
        let session_id = "deadbeef-0000-0000-0000-000000000000";
        let path = project.join(format!("{session_id}.jsonl"));
        write_jsonl(
            &path,
            &[
                "",
                "garbage not json",
                r#"{"type":"user","timestamp":"2026-03-30T04:33:28Z","cwd":"/x/y","message":{"role":"user","content":"hello"}}"#,
                "",
            ],
        );
        let sessions = scan_at(dir.path(), fixture_now()).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions.get(session_id).unwrap().title, "hello");
    }

    #[test]
    fn scan_filters_stale_sessions_at_seven_days() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("-Users-x-proj");
        fs::create_dir_all(&project).unwrap();

        // Same fixture, two files: one fresh, one stale relative to `now`.
        let now = Utc.with_ymd_and_hms(2026, 4, 10, 12, 0, 0).unwrap();
        write_jsonl(
            &project.join("fresh.jsonl"),
            &[
                r#"{"type":"user","timestamp":"2026-04-09T12:00:00Z","cwd":"/x/y","message":{"role":"user","content":"hi"}}"#,
            ],
        );
        write_jsonl(
            &project.join("stale.jsonl"),
            &[
                r#"{"type":"user","timestamp":"2026-04-02T12:00:00Z","cwd":"/x/y","message":{"role":"user","content":"old"}}"#,
            ],
        );

        let sessions = scan_at(dir.path(), now).unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions.contains_key("fresh"));
        assert!(!sessions.contains_key("stale"));
    }
}
