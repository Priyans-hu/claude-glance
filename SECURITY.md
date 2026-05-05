# Security Policy

`claude-glance` is a local-only macOS app — it makes no network calls and reads only files Claude Code already writes on your machine. Even so, security reports are welcome.

## Reporting a vulnerability

Please report security issues privately rather than opening a public issue.

**Email:** `pg.tldr@gmail.com`

Include:

- A description of the issue
- Steps to reproduce (if applicable)
- Affected version / commit SHA
- Your suggested fix, if any

You can expect an acknowledgement within 7 days. Please give a reasonable window for a fix before public disclosure.

## Supported versions

Only the latest release is supported. `claude-glance` is pre-alpha; expect frequent breaking changes.

## Scope

In scope:

- Local privilege escalation
- Reading files outside `~/.claude/projects/` and `~/.claude/settings.json`
- Any network egress from the app
- Data integrity issues that could corrupt Claude Code state

Out of scope:

- Issues in upstream dependencies (please report to the dependency maintainer)
- Social engineering / phishing
- Issues requiring physical access to the unlocked machine
