//! Parsing of Claude Code session JSONL transcripts.
//!
//! Each line in `~/.claude/projects/<encoded-cwd>/<session-uuid>.jsonl` is a
//! self-contained JSON object describing a single session event. The schema
//! is informal and varies across Claude Code versions, so we parse loosely:
//! pick out the fields we care about, ignore the rest.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Subset of an assistant message's `content[]` item type. We only care
/// whether the assistant emitted text vs. tool_use vs. thinking, so other
/// shapes collapse to `Other`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantContent {
    Text,
    ToolUse,
    Thinking,
    Other,
}

/// Normalized event shape covering only fields the watcher / status logic
/// needs. The full JSONL line is intentionally not retained.
#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub kind: EventKind,
    pub timestamp: Option<DateTime<Utc>>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub permission_mode: Option<String>,
    /// First user prompt text, only populated for `EventKind::User`.
    pub user_text: Option<String>,
    /// Assistant message content types, in order.
    pub assistant_content: Vec<AssistantContent>,
    /// Token totals on assistant turns (input + output, cache excluded).
    pub tokens: Option<u64>,
    /// `summary` line value, when applicable.
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    User,
    Assistant,
    System,
    Summary,
    FileHistorySnapshot,
    /// Anything we don't care about (progress, queue-operation, …).
    Other(String),
}

#[derive(Deserialize)]
struct RawEvent {
    #[serde(rename = "type")]
    type_: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    git_branch: Option<String>,
    #[serde(rename = "permissionMode")]
    permission_mode: Option<String>,
    message: Option<Value>,
    summary: Option<Value>,
}

/// Parse a single JSONL line into a `SessionEvent`. Unknown / malformed
/// shapes return `Ok(Other)` rather than an error so the watcher can keep
/// processing.
pub fn parse_line(line: &str) -> Result<SessionEvent, ParseError> {
    let raw: RawEvent = serde_json::from_str(line)?;
    let kind = match raw.type_.as_deref() {
        Some("user") => EventKind::User,
        Some("assistant") => EventKind::Assistant,
        Some("system") => EventKind::System,
        Some("summary") => EventKind::Summary,
        Some("file-history-snapshot") => EventKind::FileHistorySnapshot,
        Some(other) => EventKind::Other(other.to_string()),
        None => EventKind::Other(String::new()),
    };

    let timestamp = raw
        .timestamp
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let mut user_text = None;
    let mut assistant_content = Vec::new();
    let mut tokens = None;

    if matches!(kind, EventKind::User) {
        if let Some(msg) = raw.message.as_ref() {
            user_text = extract_user_text(msg);
        }
    } else if matches!(kind, EventKind::Assistant) {
        if let Some(msg) = raw.message.as_ref() {
            assistant_content = extract_assistant_content(msg);
            tokens = extract_tokens(msg);
        }
    }

    let summary = match (&kind, &raw.summary) {
        (EventKind::Summary, Some(Value::String(s))) => Some(s.clone()),
        (EventKind::Summary, Some(other)) => Some(other.to_string()),
        _ => None,
    };

    Ok(SessionEvent {
        kind,
        timestamp,
        session_id: raw.session_id,
        cwd: raw.cwd,
        git_branch: raw.git_branch.filter(|s| !s.is_empty() && s != "HEAD"),
        permission_mode: raw.permission_mode,
        user_text,
        assistant_content,
        tokens,
        summary,
    })
}

fn extract_user_text(msg: &Value) -> Option<String> {
    let content = msg.get("content")?;
    // A user message's content is sometimes a plain string, sometimes an
    // array of `{type: "text", text: "..."}` blocks.
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for item in arr {
            if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                    return Some(t.to_string());
                }
            }
        }
    }
    None
}

fn extract_assistant_content(msg: &Value) -> Vec<AssistantContent> {
    let Some(arr) = msg.get("content").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .map(|item| match item.get("type").and_then(|v| v.as_str()) {
            Some("text") => AssistantContent::Text,
            Some("tool_use") => AssistantContent::ToolUse,
            Some("thinking") => AssistantContent::Thinking,
            _ => AssistantContent::Other,
        })
        .collect()
}

fn extract_tokens(msg: &Value) -> Option<u64> {
    let usage = msg.get("usage")?;
    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    Some(input + output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_message() {
        let line = r#"{"type":"user","timestamp":"2026-03-30T04:33:28.533Z","sessionId":"abc","cwd":"/tmp/x","permissionMode":"default","message":{"role":"user","content":"hello there"}}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::User);
        assert_eq!(ev.user_text.as_deref(), Some("hello there"));
        assert_eq!(ev.session_id.as_deref(), Some("abc"));
        assert_eq!(ev.cwd.as_deref(), Some("/tmp/x"));
        assert_eq!(ev.permission_mode.as_deref(), Some("default"));
        assert!(ev.timestamp.is_some());
    }

    #[test]
    fn parses_user_message_with_array_content() {
        let line =
            r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"hi"}]}}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.user_text.as_deref(), Some("hi"));
    }

    #[test]
    fn parses_assistant_with_tool_use() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ok"},{"type":"tool_use","id":"x","name":"Read","input":{}}],"usage":{"input_tokens":10,"output_tokens":5}}}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::Assistant);
        assert_eq!(
            ev.assistant_content,
            vec![AssistantContent::Text, AssistantContent::ToolUse]
        );
        assert_eq!(ev.tokens, Some(15));
    }

    #[test]
    fn parses_assistant_text_only() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"done."}]}}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.assistant_content, vec![AssistantContent::Text]);
    }

    #[test]
    fn parses_summary() {
        let line = r#"{"type":"summary","summary":"Refactor the parser"}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::Summary);
        assert_eq!(ev.summary.as_deref(), Some("Refactor the parser"));
    }

    #[test]
    fn parses_system_event() {
        let line = r#"{"type":"system","subtype":"local_command"}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::System);
    }

    #[test]
    fn parses_file_history_snapshot() {
        let line = r#"{"type":"file-history-snapshot","messageId":"m"}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::FileHistorySnapshot);
    }

    #[test]
    fn unknown_type_becomes_other() {
        let line = r#"{"type":"progress"}"#;
        let ev = parse_line(line).unwrap();
        assert_eq!(ev.kind, EventKind::Other("progress".into()));
    }

    #[test]
    fn empty_git_branch_becomes_none() {
        let line = r#"{"type":"user","gitBranch":""}"#;
        let ev = parse_line(line).unwrap();
        assert!(ev.git_branch.is_none());
    }

    #[test]
    fn malformed_json_errors() {
        assert!(parse_line("not json").is_err());
    }
}
