# Release Process for Datamorph

This document describes how to create a new release of Datamorph.

## Prerequisites

1. **GitHub repository access** with write permissions
2. **GitHub Personal Access Token (PAT)** with `repo` scope for pushing tags and creating releases
3. **Rust toolchain** installed for building
4. **jq** installed (for release script JSON parsing)
5. **GPG** (optional) for signed tags

## Step-by-Step Release

### 1. Prepare the Release

- Ensure all features for this version are merged into `main`
- Update version in `Cargo.toml` (major.minor.patch)
- Update `README.md` with any new features
- Create `CHANGELOG.md` entry if not present
- Commit everything:
```bash
git add .
git commit -m "chore: prepare for v0.1.0"
```

### 2. Build All Binaries

The easiest way is to use the provided build script:

```bash
cd /path/to/datamorph
./scripts/build-release.sh
```

**Manual build (if script fails):**

```bash
# Linux (static)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/datamorph release/datamorph-linux-amd64

# macOS Intel (build on macOS)
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/datamorph release/datamorph-macos-amd64

# macOS ARM (Apple Silicon)
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/datamorph release/datamorph-macos-arm64

# Windows (build on Windows or WSL with MSVC)
cargo build --release --target x86_64-pc-windows-msvc
cp target/x86_64-pc-windows-msvc/release/datamorph.exe release/datamorph-windows-amd64.exe
```

**Note:** For non-Linux platforms, use native OS or CI (GitHub Actions).

### 3. Verify Binaries

```bash
# Test each binary
./release/datamorph-linux-amd64 --version
./release/datamorph-macos-amd64 --version
./release/datamorph-windows-amd64.exe --version
```

They should all print `datamorph 0.1.0`.

### 4. Create Git Tag

```bash
# Annotated tag (recommended)
git tag -a v0.1.0 -m "Datamorph v0.1.0 — Universal data format transformer"

# Push tag to GitHub
git push origin v0.1.0
```

Alternatively, sign with GPG:
```bash
git tag -s v0.1.0 -m "Datamorph v0.1.0"
```

### 5. Create GitHub Release (Option A: Automated)

Use the provided script:

```bash
export GITHUB_TOKEN=ghp_your_token_here
./scripts/create-release.sh v0.1.0 release/*
```

The script will:
1. Create a GitHub release for tag `v0.1.0`
2. Upload each file in `release/` as an asset
3. Print the release URL

### 5b. Create GitHub Release (Option B: Manual)

1. Go to https://github.com/aksaayyy/Datamorph/releases/new
2. Choose tag `v0.1.0` (or type it to create new tag)
3. Fill in release notes (see `CHANGELOG.md` or use auto-generated)
4. Drag-and-drop all files from `release/` directory
5. Click "Publish release"

### 6. Post-Release

- **Update package manager indexes** (if applicable):
  - Homebrew: `brew tap-new aksaayyy/tap && brew create ...` (out of scope)
  - Cargo crates.io: Not applicable (not published there)
- **Announce** on:
  - Twitter/X
  - Reddit r/rust, r/devops, r/dataengineering
  - Hacker News
  - Relevant Discord/Slack communities

### 7. Verify Deployment

```bash
# Test installation script
curl -fsSL https://raw.githubusercontent.com/aksaayyy/Datamorph/main/scripts/install.sh | bash -s -- --verify

# Test binary download
curl -fL https://github.com/aksaayyy/Datamorph/releases/download/v0.1.0/datamorph-linux-amd64 -o /tmp/datamorph
chmod +x /tmp/datamorph
/tmp/datamorph --version
```

---

## Troubleshooting

### Build fails on Windows: "link.exe not found"

You're using the MSVC target without Visual Studio Build Tools. Either:
- Install Visual Studio Build Tools with C++ workload, or
- Use GNU target: `x86_64-pc-windows-gnu` (requires mingw-w64)

### GitHub Actions builds macOS but fails on codesigning

GitHub Actions macOS runners don't need codesigning for release builds unless you're distributing outside GitHub. Add `--locked` to cargo if you want reproducible builds.

### Release assets missing

If `create-release.sh` fails to upload:
- Verify `GITHUB_TOKEN` has `repo` scope
- Check each file exists and is readable
- Ensure tag already exists on GitHub before uploading assets

### "Bad credentials" error

Double-check your PAT. Classic tokens starting with `ghp_` are correct. Fine-grained tokens (starting with `github_pat_`) don't work for Git operations; use a classic PAT for both Git and API.

---

## Quick Reference Commands

```bash
# Full release (Linux only, automated)
./scripts/build-release.sh
export GITHUB_TOKEN=xxx
./scripts/create-release.sh v0.1.0 release/*

# Tag only
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

*Last updated: 2026-04-24*
