# Contributing

Thanks for your interest in `claude-glance`. The project is pre-alpha — APIs, file layout, and UI are all in flux.

## Development setup

Requires:

- [Bun](https://bun.sh) (latest)
- [Rust](https://rustup.rs/) stable + `rustfmt`, `clippy`
- Xcode Command Line Tools (`xcode-select --install`)

```bash
git clone https://github.com/Priyans-hu/claude-glance.git
cd claude-glance
bun install
bun tauri dev
```

## Pull requests

- Branch off `main`. Use prefixes: `feat/`, `fix/`, `chore/`, `docs/`, `refactor/`, `test/`.
- Keep commits small and focused. Squash-merge will collapse them at the end if appropriate.
- Before pushing, run:
  ```bash
  bun run check       # svelte-check / type check
  bun run lint        # eslint / prettier
  bun test            # frontend tests
  cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test
  ```
- All CI checks must pass before merge.
- PR titles: imperative mood ("add X", "fix Y").

## Bug reports

Open an issue using the **Bug** template. Include:

- macOS version (`sw_vers -productVersion`)
- `claude-glance` version (Help → About, or commit SHA)
- Reproduction steps
- A redacted JSONL snippet if a parser bug

## Feature requests

Open an issue using the **Feature** template. Describe the use case before proposing an implementation.

## Code style

- **Rust:** `cargo fmt`, `clippy` clean. Prefer `?` over `unwrap` in non-test code.
- **TypeScript / Svelte:** strict mode on. Avoid `any`. Prefer derived stores over manual subscriptions.
- **Comments:** explain _why_, not _what_. Skip if the code is self-evident.

## Releases

Releases are cut from `main`. Each release builds an unsigned `.dmg` artifact via the `release.yml` workflow. SemVer applies once we hit `0.1.0`; until then expect breaking changes on any commit.
