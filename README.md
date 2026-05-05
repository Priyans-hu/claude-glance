# claude-glance

Read-only macOS dashboard for monitoring all your active Claude Code sessions at a glance.

> **Status:** pre-alpha — under active development.

## What it does

`claude-glance` watches `~/.claude/projects/` for active [Claude Code](https://claude.com/claude-code) sessions and surfaces them in a single window grouped by status: **waiting · active · plan · idle · done**. Designed to live on a second monitor.

- All-local, no network calls, no telemetry
- Event-driven via FSEvents — near-zero CPU when idle
- Reads JSONL transcripts that Claude Code already writes; no hooks required
- Native macOS app (Tauri) — light footprint (~10 MB binary, ~30 MB RAM)

## Screenshot

_Coming once the UI lands._

## Installation

`claude-glance` is **not** distributed via the App Store and the build is **not signed** with an Apple Developer certificate. Two install paths.

### Option 1 — Build from source (recommended)

Requires [Bun](https://bun.sh), [Rust](https://rustup.rs/) (stable), and Xcode Command Line Tools.

```bash
git clone https://github.com/Priyans-hu/claude-glance.git
cd claude-glance
bun install
bun tauri build
```

The built app lands in `src-tauri/target/release/bundle/macos/claude-glance.app`. Move it to `/Applications`.

### Option 2 — Pre-built binary

Download the latest `.dmg` from [Releases](https://github.com/Priyans-hu/claude-glance/releases). Because the binary is unsigned, macOS Gatekeeper will block it on first launch. Bypass quarantine with:

```bash
xattr -d com.apple.quarantine /Applications/claude-glance.app
```

Then open the app normally.

> A Homebrew tap is planned — track [#1](https://github.com/Priyans-hu/claude-glance/issues).

## Development

```bash
bun install
bun tauri dev
```

## How it works

1. Rust backend uses `notify` (FSEvents on macOS) to watch `~/.claude/projects/**/*.jsonl`.
2. New writes are parsed line-by-line; session state is derived from event recency + last event type.
3. Frontend (Svelte) renders a grouped, scannable list with PR/git symbols and subagent counts.

No polling. No daemons. No network.

## Privacy

Everything stays on your machine. `claude-glance` reads files Claude Code already writes; it makes no network calls. Source is auditable.

## License

[MIT](./LICENSE)
