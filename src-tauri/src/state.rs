//! Session state derivation: turn a stream of `SessionEvent`s plus some
//! filesystem context (filename, subagent dir) into the `Session` shape the
//! UI consumes.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::parser::{AssistantContent, EventKind, SessionEvent};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Working,
    Waiting,
    Plan,
    Idle,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub cwd: String,
    pub project: String,
    pub branch: Option<String>,
    pub title: String,
    pub status: SessionStatus,
    #[serde(rename = "currentTool")]
    pub current_tool: Option<String>,
    #[serde(rename = "permissionMode")]
    pub permission_mode: Option<String>,
    #[serde(rename = "subagentCount")]
    pub subagent_count: u32,
    #[serde(rename = "lastActivity")]
    pub last_activity: String,
    pub tokens: u64,
    /// Absolute path to the JSONL transcript backing this session. Carried
    /// on the Session so commands like stop / delete don't have to re-walk
    /// the projects root every time.
    #[serde(rename = "transcriptPath")]
    pub transcript_path: PathBuf,
}

const TITLE_MAX_CHARS: usize = 80;
const WORKING_USER_MSG_WINDOW_SECS: i64 = 60;
const WORKING_TOOL_WINDOW_SECS: i64 = 30;
const ATTENTION_WINDOW_SECS: i64 = 2 * 60 * 60;

/// Derive a status from events sorted ascending by timestamp.
/// `now` is injected for testability.
pub fn derive_status(events: &[SessionEvent], now: DateTime<Utc>) -> SessionStatus {
    // Prefer events with timestamps; fall back to last event with one.
    let last_ts = events.iter().rev().find_map(|e| e.timestamp);
    let Some(last_ts) = last_ts else {
        return SessionStatus::Idle;
    };

    let age_secs = (now - last_ts).num_seconds();
    let last_event_with_ts = events.iter().rev().find(|e| e.timestamp.is_some());
    let last_perm = events
        .iter()
        .rev()
        .find_map(|e| e.permission_mode.as_deref());
    let last_assistant = events
        .iter()
        .rev()
        .find(|e| matches!(e.kind, EventKind::Assistant));

    // 1) Working: a fresh user message means Claude received the prompt and is
    //    generating; the assistant turn just hasn't been flushed to the JSONL
    //    yet. Without this the session shows up as "waiting on you".
    if age_secs < WORKING_USER_MSG_WINDOW_SECS {
        if let Some(ev) = last_event_with_ts {
            if matches!(ev.kind, EventKind::User) {
                return SessionStatus::Working;
            }
        }
    }

    // 2) Working: very recent, last assistant emitted a tool_use or thinking.
    if age_secs < WORKING_TOOL_WINDOW_SECS {
        if let Some(ev) = last_assistant {
            if ev.assistant_content.contains(&AssistantContent::ToolUse)
                || ev.assistant_content.contains(&AssistantContent::Thinking)
            {
                return SessionStatus::Working;
            }
        }
    }

    // 3) Plan mode wins if within attention window.
    if matches!(last_perm, Some("plan")) && age_secs < ATTENTION_WINDOW_SECS {
        return SessionStatus::Plan;
    }

    // 4) Waiting: within attention window, last assistant message was text only.
    if age_secs < ATTENTION_WINDOW_SECS {
        if let Some(ev) = last_assistant {
            let has_tool = ev.assistant_content.contains(&AssistantContent::ToolUse);
            let has_text = ev.assistant_content.contains(&AssistantContent::Text);
            if !has_tool && has_text {
                return SessionStatus::Waiting;
            }
        }
    }

    // 5) Otherwise idle.
    SessionStatus::Idle
}

/// Build a `Session` for the given session id from its events. Caller is
/// expected to have parsed all lines of the `.jsonl` and resolved the
/// subagent count (number of `subagents/agent-*.jsonl` files).
pub fn build_session(
    session_id: String,
    events: &[SessionEvent],
    subagent_count: u32,
    now: DateTime<Utc>,
    transcript_path: PathBuf,
) -> Session {
    // Pick the canonical cwd / branch / permission_mode from the most recent
    // event that has them set.
    let cwd = events
        .iter()
        .rev()
        .find_map(|e| e.cwd.clone())
        .unwrap_or_default();
    let branch = events.iter().rev().find_map(|e| e.git_branch.clone());
    let permission_mode = events.iter().rev().find_map(|e| e.permission_mode.clone());

    let project = derive_project(&cwd);
    let title = derive_title(events, &project);
    let status = derive_status(events, now);

    let last_activity = events
        .iter()
        .rev()
        .find_map(|e| e.timestamp)
        .map(|t| t.to_rfc3339())
        .unwrap_or_default();

    let tokens: u64 = events.iter().filter_map(|e| e.tokens).sum();

    // Tool name is not preserved through `parse_line` yet. Pass-through for
    // v2; the running-icon glow is enough signal in the UI for now.
    let current_tool = None;

    Session {
        id: session_id,
        cwd,
        project,
        branch,
        title,
        status,
        current_tool,
        permission_mode,
        subagent_count,
        last_activity,
        tokens,
        transcript_path,
    }
}

fn derive_project(cwd: &str) -> String {
    if cwd.is_empty() {
        return String::new();
    }
    // Worktree paths look like
    //   /Users/x/proj/--worktrees-fix-foo
    //   /Users/x/proj/.claude/worktrees/fix-foo
    let parts: Vec<&str> = cwd.trim_end_matches('/').split('/').collect();
    let last = parts.last().copied().unwrap_or("");

    // Case A: `repo--worktrees-<name>` style (path encoded with double dashes).
    if let Some(idx) = cwd.find("--worktrees-") {
        let before = &cwd[..idx];
        let repo = before
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let after = &cwd[idx + "--worktrees-".len()..];
        // Stop at the next slash if any.
        let wt = after.split('/').next().unwrap_or("");
        if !repo.is_empty() && !wt.is_empty() {
            return format!("{repo}:{wt}");
        }
    }

    // Case B: `<repo>/.claude/worktrees/<name>` style.
    if let Some(idx) = cwd.find("/.claude/worktrees/") {
        let before = &cwd[..idx];
        let repo = before
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let after = &cwd[idx + "/.claude/worktrees/".len()..];
        let wt = after.split('/').next().unwrap_or("");
        if !repo.is_empty() && !wt.is_empty() {
            return format!("{repo}:{wt}");
        }
    }

    last.to_string()
}

fn derive_title(events: &[SessionEvent], project: &str) -> String {
    // 1) summary line (rare but exists in some CC versions).
    if let Some(s) = events.iter().find_map(|e| e.summary.clone()) {
        return truncate(&s, TITLE_MAX_CHARS);
    }
    // 2) first user message text.
    if let Some(t) = events
        .iter()
        .find(|e| matches!(e.kind, EventKind::User))
        .and_then(|e| e.user_text.clone())
    {
        return truncate(t.trim(), TITLE_MAX_CHARS);
    }
    // 3) fallback.
    if !project.is_empty() {
        format!("{project} session")
    } else {
        "untitled session".to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_line;
    use chrono::TimeZone;

    fn ev_at(now: DateTime<Utc>, secs_ago: i64, line: &str) -> SessionEvent {
        let ts = now - chrono::Duration::seconds(secs_ago);
        let mut e = parse_line(line).unwrap();
        e.timestamp = Some(ts);
        e
    }

    fn t() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
    }

    fn fake_path(id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{id}.jsonl"))
    }

    #[test]
    fn plan_mode_recent_is_plan() {
        let now = t();
        let events = vec![
            ev_at(
                now,
                5,
                r#"{"type":"user","permissionMode":"plan","message":{"role":"user","content":"x"}}"#,
            ),
            ev_at(
                now,
                5,
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"plan ok"}]}}"#,
            ),
        ];
        assert_eq!(derive_status(&events, now), SessionStatus::Plan);
    }

    #[test]
    fn very_recent_tool_use_is_working() {
        let now = t();
        let events = vec![ev_at(
            now,
            5,
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","id":"x","input":{}}]}}"#,
        )];
        assert_eq!(derive_status(&events, now), SessionStatus::Working);
    }

    #[test]
    fn recent_text_only_is_waiting() {
        let now = t();
        let events = vec![ev_at(
            now,
            30,
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"done"}]}}"#,
        )];
        assert_eq!(derive_status(&events, now), SessionStatus::Waiting);
    }

    #[test]
    fn old_session_outside_attention_window_is_idle() {
        let now = t();
        let events = vec![ev_at(
            now,
            3 * 60 * 60,
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"done"}]}}"#,
        )];
        assert_eq!(derive_status(&events, now), SessionStatus::Idle);
    }

    #[test]
    fn no_events_with_timestamp_is_idle() {
        let events: Vec<SessionEvent> = vec![];
        assert_eq!(derive_status(&events, t()), SessionStatus::Idle);
    }

    // Regression test for the v0.0.1 misclassification bug: a session whose
    // last event is a fresh `user` message (Claude is generating, transcript
    // hasn't flushed) was incorrectly shown as "waiting on you".
    #[test]
    fn fresh_user_msg_is_working() {
        let now = t();
        let events = vec![ev_at(
            now,
            5,
            r#"{"type":"user","message":{"role":"user","content":"hi"}}"#,
        )];
        assert_eq!(derive_status(&events, now), SessionStatus::Working);
    }

    #[test]
    fn fresh_user_msg_at_window_boundary_is_idle() {
        // 61s old user message — outside the 60s working window, no assistant
        // turn yet, so the session falls through to idle.
        let now = t();
        let events = vec![ev_at(
            now,
            61,
            r#"{"type":"user","message":{"role":"user","content":"hi"}}"#,
        )];
        assert_eq!(derive_status(&events, now), SessionStatus::Idle);
    }

    #[test]
    fn fresh_user_msg_overrides_stale_assistant() {
        // The bug case in full: an assistant text-only turn from earlier, then
        // the user replies; the assistant turn for the new prompt hasn't been
        // appended yet. Status must read as Working, not Waiting.
        let now = t();
        let events = vec![
            ev_at(
                now,
                120,
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"done"}]}}"#,
            ),
            ev_at(
                now,
                3,
                r#"{"type":"user","message":{"role":"user","content":"next"}}"#,
            ),
        ];
        assert_eq!(derive_status(&events, now), SessionStatus::Working);
    }

    #[test]
    fn build_session_extracts_title_from_first_user_msg() {
        let now = t();
        let events = vec![
            ev_at(
                now,
                100,
                r#"{"type":"user","cwd":"/Users/x/myproj","gitBranch":"main","permissionMode":"default","message":{"role":"user","content":"refactor the parser"}}"#,
            ),
            ev_at(
                now,
                90,
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ok"}]}}"#,
            ),
        ];
        let s = build_session("sess-1".into(), &events, 0, now, fake_path("sess-1"));
        assert_eq!(s.id, "sess-1");
        assert_eq!(s.project, "myproj");
        assert_eq!(s.title, "refactor the parser");
        assert_eq!(s.branch.as_deref(), Some("main"));
        assert_eq!(s.permission_mode.as_deref(), Some("default"));
    }

    #[test]
    fn build_session_handles_worktree_dashes() {
        let now = t();
        let events = vec![ev_at(
            now,
            5,
            r#"{"type":"user","cwd":"/Users/x/plivo/contacto-console--worktrees-fix-cdr-export-tz"}"#,
        )];
        let s = build_session("s".into(), &events, 0, now, fake_path("s"));
        assert_eq!(s.project, "contacto-console:fix-cdr-export-tz");
    }

    #[test]
    fn build_session_handles_dot_claude_worktree() {
        let now = t();
        let events = vec![ev_at(
            now,
            5,
            r#"{"type":"user","cwd":"/Users/x/plivo/contacto-core/.claude/worktrees/fix-buddy-escalation-ip"}"#,
        )];
        let s = build_session("s".into(), &events, 0, now, fake_path("s"));
        assert_eq!(s.project, "contacto-core:fix-buddy-escalation-ip");
    }

    #[test]
    fn build_session_truncates_long_titles() {
        let now = t();
        let long = "a".repeat(200);
        let line = format!(
            r#"{{"type":"user","cwd":"/x/y","message":{{"role":"user","content":"{long}"}}}}"#
        );
        let events = vec![ev_at(now, 5, &line)];
        let s = build_session("s".into(), &events, 0, now, fake_path("s"));
        assert!(s.title.chars().count() <= 81); // 80 chars + ellipsis
        assert!(s.title.ends_with('…'));
    }

    #[test]
    fn build_session_subagent_count_propagates() {
        let now = t();
        let events = vec![ev_at(
            now,
            10,
            r#"{"type":"user","cwd":"/x/proj","message":{"role":"user","content":"go"}}"#,
        )];
        let s = build_session("s".into(), &events, 3, now, fake_path("s"));
        assert_eq!(s.subagent_count, 3);
    }

    #[test]
    fn build_session_sums_tokens() {
        let now = t();
        let events = vec![
            ev_at(
                now,
                100,
                r#"{"type":"user","cwd":"/x/proj","message":{"role":"user","content":"go"}}"#,
            ),
            ev_at(
                now,
                90,
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"a"}],"usage":{"input_tokens":10,"output_tokens":5}}}"#,
            ),
            ev_at(
                now,
                80,
                r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"b"}],"usage":{"input_tokens":3,"output_tokens":2}}}"#,
            ),
        ];
        let s = build_session("s".into(), &events, 0, now, fake_path("s"));
        assert_eq!(s.tokens, 20);
    }

    #[test]
    fn build_session_propagates_transcript_path() {
        let now = t();
        let path = PathBuf::from("/Users/x/.claude/projects/-Users-x-proj/abc.jsonl");
        let events = vec![ev_at(
            now,
            10,
            r#"{"type":"user","cwd":"/x/proj","message":{"role":"user","content":"go"}}"#,
        )];
        let s = build_session("abc".into(), &events, 0, now, path.clone());
        assert_eq!(s.transcript_path, path);
    }
}
