# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3]

### Added

- Each session row now exposes a 3-dot overflow menu (lucide `MoreVertical`) revealed on row hover with two actions: **Stop session** sends `SIGTERM` to the `claude` process holding the JSONL (escalates to `SIGKILL` after 300ms) and leaves the transcript intact. **Delete session…** prompts via the native confirm dialog and, on confirmation, kills the process if alive then removes both the JSONL and the per-session sibling directory (`subagents/`, `tool-results/`, `session-env/`).
- New Rust modules: `process` (finds the PID holding a `.jsonl` open via `lsof -F p` and kills it via `nix::sys::signal`) and `cleanup` (best-effort recursive removal of the JSONL and its sibling directory).
- New Tauri commands: `stop_session(id)` returns whether a process was actually stopped; `delete_session(id)` is fire-and-forget but emits a `sessions_changed` event after mutating the map.
- `Session.transcript_path` (camelCase `transcriptPath` over the wire) so the new commands can resolve the JSONL without re-walking `~/.claude/projects/`.

### Fixed

- `rescan_sessions` now returns the fresh `Vec<Session>` directly and the frontend applies it to the store immediately, sidestepping the `sessions_changed` event-listener race that occasionally caused refreshes to look like no-ops.
- Notify debouncer now polls every 5 seconds as a safety net for missed FSEvents — newly-created sessions in busy directories no longer slip through the watcher.

## [0.0.2]

### Fixed

- Sessions where the user has just submitted a message (and Claude is generating a reply that hasn't yet been written to the JSONL transcript) were being shown as `WAITING ON YOU`. They are now correctly classified as `WORKING` whenever the last event is a `user` message within the past 60s.

### Added

- Scanner-level filter that drops JSONL files whose latest event timestamp is more than 7 days old. Sessions in the 2-7 day window are folded into a single collapsed `RECENT — last 7 days` group whose expanded state is persisted in `localStorage` under `cg.showRecent`.
- Click a session row to toggle a detail strip showing the full session UUID (mono, with `[ copy ]`), `cwd`, git branch, started timestamp, and a `[ copy resume command ]` button that copies `claude --resume <uuid>` to the clipboard.
- Right-click a session row for a vanilla-CSS context menu with the same two copy actions.
- Header refresh button (lucide `RefreshCw`) that calls a new `rescan_sessions` Tauri command, spins for at least 400ms so it doesn't flicker, then briefly shows `Re-scanned N sessions`.

### Changed

- `SessionStatus` is trimmed from six variants to four (`working`, `waiting`, `plan`, `idle`); `done` and `error` were unused in derivation. Bucketing for display is now driven by event age rather than status alone.
- Status icons updated to match the new label set: `Activity` (working), `CircleAlert` (waiting), `BookOpen` (plan), `Clock` (idle), `Archive` (recent).

## [0.0.1]

### Added

- Initial project scaffolding (Tauri 2 + SvelteKit + Tailwind v4).
- FSEvents-based watcher and JSONL parser for `~/.claude/projects/`.
- Status-grouped UI (`running` / `waiting` / `plan` / `idle` / `done` / `error`).
