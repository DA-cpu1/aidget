# Releasing aidget

## Release contract

Releases are built and published by `.github/workflows/release.yml` from an
annotated `vMAJOR.MINOR.PATCH` tag in `aidget-ai/aidget`.

- The tag must point to a reviewed commit that is already on `main`.
- The version must be strictly newer than the current latest release.
- No source version bump commit is created.
- The workflow injects the tag version into `Cargo.toml`, `package.json` and
  `tauri.conf.json` before checks and packaging.
- The workflow stages a draft release only after every platform builds, then
  publishes it as the latest stable release once all assets and notes exist.

## Required repository secret

Configure `DEEPSEEK_API_KEY` as a repository Actions secret. It is used only by
the release notes job: first to generate an English changelog from the commit
subjects since the previous release, then to translate that changelog into
Simplified Chinese.

There are no signing or updater secrets in the current process.

## Create a release

Start from current `main` and confirm the reviewed commit is part of it:

```bash
git switch main
git pull --ff-only
git fetch --tags origin

REVIEWED_SHA=<reviewed-main-commit>
git merge-base --is-ancestor "$REVIEWED_SHA" origin/main
```

Choose the next version and confirm it is newer than the latest published tag:

```bash
TAG=v0.2.0
LATEST=$(gh api repos/aidget-ai/aidget/releases/latest --jq .tag_name 2>/dev/null || true)
if [ -n "$LATEST" ]; then
  node scripts/release/version.mjs newer "$TAG" "$LATEST"
fi
```

Create and push an annotated tag on the reviewed commit:

```bash
git tag -a "$TAG" "$REVIEWED_SHA" -m "aidget $TAG"
git push origin "$TAG"
```

Watch the Release workflow:

```bash
gh run watch --repo aidget-ai/aidget
```

## Release assets

Every successful release contains:

- `aidget_<version>_aarch64.dmg`
- `aidget_<version>_x64.dmg`
- `aidget_<version>_x64-setup.exe`
- `aidget_<version>_amd64.AppImage`
- `aidget_<version>_amd64.deb`
- `aidget_<version>_amd64.rpm`
- `SHA256SUMS`

DeepSeek generates the English notes from commit subjects between the previous
and current release tags. A second DeepSeek call adds a Simplified Chinese
translation below `<!-- lang:zh -->`.

## Failure and retry rules

- A missing or failed `DEEPSEEK_API_KEY` fails the release before a public
  release is created. Fix the secret and rerun the workflow.
- A failed build leaves no public release. A transient runner or service failure
  may be retried on the same unpublished tag.
- A failure that needs a code or workflow change requires a new commit on
  `main` and a new patch tag. Never move or delete the failed tag.
- A draft release left by a partial publish is replaced on rerun. A published
  release is immutable and cannot be replaced by this workflow.
- Never hand-upload or replace assets after a release is public.

## Signing and updates

The current macOS and Windows artifacts are unsigned. macOS users may see
Gatekeeper warnings, and Windows users may see SmartScreen warnings.

No client auto-update is included: there is no Tauri updater plugin,
`latest.json`, update public key or in-app update check.

The first live release is also the first end-to-end test of GitHub-hosted macOS,
Windows and Linux packaging. Until it runs, treat those packaging paths as
unverified.
