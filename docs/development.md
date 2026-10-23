# Development workflow

## Requirements

- Rust 1.85 or newer
- Node.js 20 or newer; CI uses Node.js 24
- pnpm 12
- The platform WebView and packaging dependencies described in the README

## Pull requests

Open a pull request only after a required Contribution Proposal has been
accepted with `lgtmi` and recorded in `.github/APPROVED_CONTRIBUTORS`, and the
change has a reproducible verification record. CI and a reviewer other than the
author must both accept the current head.

Do not merge by moving a branch after review. If the head changes, run the
relevant checks again and merge the newly reviewed SHA.

## Contributor approval automation

`main` only accepts direct pushes from the repository owner and the
`aidget-bot` GitHub App. The App is installed only on this
repository with Contents write access. The approval workflow uses the
`AIDGET_BOT_CLIENT_ID` and `AIDGET_BOT_APP_ID` repository variables plus the
`AIDGET_BOT_PRIVATE_KEY` Actions secret to mint a short-lived installation token.


## Local checks

```bash
pnpm install --frozen-lockfile
pnpm test
pnpm typecheck
cargo fmt --check
AIDGET_CHECK_TARGET=1 cargo check --lib
cargo test
```

The cross-target library check compiles platform-gated Rust paths without the
platform bundler. It is not a runtime test. Name any macOS, Windows, Linux,
X11, Wayland, or packaging checks that were actually run.

Run the app against a throwaway data directory:

```bash
AIDGET_HOME=/tmp/aidget-dev cargo run -- ui --background
AIDGET_HOME=/tmp/aidget-dev cargo run -- doctor
```

Never use a real agent configuration or `~/.aidget` as test data.

## Version model

The checked-in version files describe development builds:

- `Cargo.toml`
- `package.json`
- `tauri.conf.json`

Release artifacts do not commit a version bump. The release workflow checks out
the reviewed tag and runs `scripts/release/version.mjs set vX.Y.Z`, which updates
those manifests only in the workflow workspace before compiling, testing and
bundling.

This means a `vX.Y.Z` release reports `X.Y.Z` from the CLI, doctor, MCP server,
main window and bundle, while ordinary source builds keep the checked-in
development version.

## CI

The reusable Check workflow runs on `main` pushes, pull requests, and from the
Release workflow:

- macOS: complete Rust tests plus the cross-target library check
- Linux: UI and repository script tests, type checks, formatting and the
  cross-target library check
- Windows: the cross-target library check

The release workflow also performs real Tauri packaging on all four release
targets.
