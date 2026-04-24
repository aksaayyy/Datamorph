# Datamorph

**Universal data format transformer CLI** — convert, query, validate, diff & repair JSON, YAML, TOML, CSV with ease.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

## Features

- **Convert** between JSON, YAML, TOML, CSV with optional pretty-printing
- **Query** data using Universal Path Language (UPL) — similar to jq but across formats
- **Validate** input data against schemas (JSON Schema planned)
- **Repair** common formatting issues (missing commas, brackets)
- **Lint** and auto-fix common problems
- **Diff** two data files, highlighting structural differences
- Zero-config auto-detection of input formats by content
- Streaming support for large files
- Colorful terminal output with optional no-color mode

## Installation

### Pre-built binaries

Download from the [releases page](https://github.com/aksaayyy/Datamorph/releases).

### Build from source

Requires Rust 1.70+:

```bash
git clone https://github.com/aksaayyy/Datamorph.git
cd Datamorph
cargo build --release
# binary at target/release/datamorph
```

## Quick start

### Convert

```bash
# JSON → YAML
datamorph convert input.json --to yaml -o output.yaml

# Pretty-print TOML
cat config.toml | datamorph convert --from toml --to toml --pretty - > pretty.toml

# CSV → JSON
datamorph convert data.csv --to json
```

### Query

```bash
# Get all names from a nested array
datamorph query data.json "users[*].name" --format yaml

# Filter users by age
datamorph query data.json "users[?age>30]" --format json
```

### Validate & Repair

```bash
datamorph validate data.json --schema schema.json

# Auto-fix common issues
datamorph repair corrupt.json --output fixed.json
```

### Diff

```bash
datamorph diff old.json new.json
```

### Lint

```bash
datamorph lint *.yaml --fix
```

## Universal Path Language (UPL)

UPL lets you navigate nested data structures:

| Syntax | Meaning |
|--------|---------|
| `.field` | Access object field |
| `[0]` | Index into array |
| `[*]` | Wildcard — all elements |
| `[?age>30]` | Filter array by condition |
| `users[?active==true].name` | Chain filter + field |

Examples:
- `users[0].email` → first user's email
- `items[*].price` → array of all prices
- `orders[?total>100].id` → IDs of expensive orders

## Supported formats

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| JSON   | ✅   | ✅    | Full support |
| YAML   | ✅   | ✅    | Preserves comments? (no) |
| TOML   | ✅   | ✅    | Round-trip |
| CSV    | ✅   | ✅    | Header detection, type inference |

## Roadmap

- [ ] JSON Schema validation
- [ ] XML support
- [ ] SQLite output
- [ ] In-place editing with backups
- [ ] Batch processing with glob patterns
- [ ] Progress bars for large files

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT © 2026 Akshay
