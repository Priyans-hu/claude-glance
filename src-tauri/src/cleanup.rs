//! Filesystem deletion for a session: the `.jsonl` transcript and the
//! sibling per-session directory (subagents, tool-results, session-env).
//!
//! Best-effort: missing sibling dirs are not an error, but a missing JSONL
//! is reported so the caller can surface it.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

/// Remove a session's JSONL transcript and its sibling per-session
/// directory. The sibling dir is `parent / file_stem` — e.g.,
/// `~/.claude/projects/-Users-x-proj/abc.jsonl` has sibling
/// `~/.claude/projects/-Users-x-proj/abc/`.
pub fn delete_session_files(jsonl: &Path) -> Result<()> {
    let parent = jsonl
        .parent()
        .ok_or_else(|| anyhow!("jsonl path has no parent: {}", jsonl.display()))?;
    let stem = jsonl
        .file_stem()
        .ok_or_else(|| anyhow!("jsonl path has no file stem: {}", jsonl.display()))?;
    let sibling_dir = parent.join(stem);

    if sibling_dir.exists() {
        fs::remove_dir_all(&sibling_dir)
            .with_context(|| format!("remove_dir_all {}", sibling_dir.display()))?;
    }

    if jsonl.exists() {
        fs::remove_file(jsonl).with_context(|| format!("remove_file {}", jsonl.display()))?;
    } else {
        return Err(anyhow!("jsonl not found: {}", jsonl.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn deletes_jsonl_when_no_sibling_exists() {
        let dir = tempdir().unwrap();
        let jsonl = dir.path().join("abc.jsonl");
        let mut f = File::create(&jsonl).unwrap();
        writeln!(f, "{{}}").unwrap();

        delete_session_files(&jsonl).unwrap();
        assert!(!jsonl.exists());
    }

    #[test]
    fn deletes_jsonl_and_sibling_dir() {
        let dir = tempdir().unwrap();
        let jsonl = dir.path().join("abc.jsonl");
        File::create(&jsonl).unwrap();
        let sibling = dir.path().join("abc");
        fs::create_dir_all(sibling.join("subagents")).unwrap();
        fs::create_dir_all(sibling.join("tool-results")).unwrap();
        File::create(sibling.join("subagents").join("agent-1.jsonl")).unwrap();
        File::create(sibling.join("tool-results").join("call-1.json")).unwrap();

        delete_session_files(&jsonl).unwrap();
        assert!(!jsonl.exists());
        assert!(!sibling.exists());
    }

    #[test]
    fn errors_when_jsonl_missing() {
        let dir = tempdir().unwrap();
        let jsonl = dir.path().join("ghost.jsonl");
        let err = delete_session_files(&jsonl).unwrap_err();
        assert!(err.to_string().contains("jsonl not found"));
    }

    #[test]
    fn deletes_sibling_dir_even_if_only_dir_exists_then_errors() {
        // Sibling dir gets cleared, then the JSONL-missing branch reports
        // the error. Documents current behaviour.
        let dir = tempdir().unwrap();
        let jsonl = dir.path().join("abc.jsonl");
        let sibling = dir.path().join("abc");
        fs::create_dir_all(&sibling).unwrap();
        File::create(sibling.join("note.txt")).unwrap();

        let err = delete_session_files(&jsonl).unwrap_err();
        assert!(err.to_string().contains("jsonl not found"));
        assert!(!sibling.exists());
    }
}
