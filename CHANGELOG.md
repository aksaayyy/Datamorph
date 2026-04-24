# Changelog

All notable changes to Datamorph will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-04-24

### Added
- Initial release of Datamorph
- **CLI Commands:**
  - `convert` — Convert between JSON, YAML, TOML, CSV
  - `query` — Universal Path Language (UPL) query engine
  - `validate` — Syntax validation (JSON Schema planned)
  - `repair` — Heuristic fixes for malformed JSON
  - `diff` — Structural diff between two files
  - `lint` — Check files, optionally fix

- **Format Support:**
  - JSON (full read/write, pretty-print)
  - YAML (read/write via serde_yaml)
  - TOML (read/write)
  - CSV (type inference, header detection, flexible parsing)

- **Query Language (UPL):**
  - Field access: `.field`, `field`
  - Array indexing: `[0]`, `[#]`
  - Wildcards: `[*]`
  - Filters: `[?age>30]`, `[?status=="active"]`
  - Comparison operators: `==`, `!=`, `>`, `<`, `>=`, `<=`

- **Error Handling:**
  - Rich error messages via `miette` + `thiserror`
  - Format-specific error sources
  - Exit codes: 0 success, 1 error

- **Documentation:**
  - README with quick start
  - CONTRIBUTING, CODE_OF_CONDUCT
  - Technical architecture doc
  - Usage guide
  - Release process docs

- **Developer Experience:**
  - Cross-platform builds (Linux static, macOS, Windows via CI)
  - One-line install script (`install.sh`)
  - GitHub Actions workflow for automated releases
  - Comprehensive `.gitignore` for Rust projects

### Changed
- N/A (initial release)

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- No remote network access (local only)
- Safe parsing with well-maintained crates (serde, csv)

---

## [Unreleased]

### Planned for v0.2.0

- JSON Schema validation support
- Streaming CSV (low memory footprint)
- Progress bars for long operations
- `--in-place` with custom backup extension
- `--output-dir` for batch conversion
- `format` subcommand to detect format of a file
- `shell` completion (bash/zsh/fish)

### Planned for v0.3.0

- Extended UPL: arithmetic (`+`, `-`), string ops (`contains`, `split`)
- Functions: `map`, `reduce`, `sort`
- Pipes: `users | map(.name)`
- Variables and custom expressions
- XML support (read-only initially)

### Planned for v1.0.0

- SQLite output adapter
- Full test coverage (>90%)
- Benchmark suite
- Fuzzing (proptest / cargo-fuzz)
- Official Debian/Homebrew packages

---

*This changelog is generated from commit messages and the project roadmap.*
