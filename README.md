# Datamorph

Universal data format transformer CLI -- convert, query, validate, diff, and repair JSON, YAML, TOML, and CSV.

## Features

- **Convert** between JSON, YAML, TOML, CSV with optional pretty-printing
- **Query** data using Universal Path Language (UPL) -- similar to jq but works across formats
- **Validate** input data against JSON Schema
- **Repair** common formatting issues (missing commas, brackets)
- **Lint** and auto-fix common problems
- **Diff** two data files, highlighting structural differences
- Auto-detection of input formats by content or file extension
- Streaming support for large files
- Colorful terminal output with optional no-color mode

## Installation

### One-line install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/aksaayyy/Datamorph/main/install.sh | bash
```

### Manual install

```bash
curl -fsSL https://github.com/aksaayyy/Datamorph/releases/latest/download/datamorph-linux-amd64 -o ~/.local/bin/datamorph
chmod +x ~/.local/bin/datamorph
```

### Build from source

Requires Rust 1.70+:

```bash
git clone https://github.com/aksaayyy/Datamorph.git
cd Datamorph
cargo build --release
# binary at target/release/datamorph
```

## Quick Start

### Convert

```bash
# JSON to YAML
datamorph convert input.json --to yaml -o output.yaml

# Pretty-print TOML
cat config.toml | datamorph convert --from toml --to toml --pretty - > pretty.toml

# CSV to JSON
datamorph convert data.csv --to json
```

### Query

```bash
# Get all names from a nested array
datamorph query data.json "users[*].name" --format yaml

# Filter users by age
datamorph query data.json "users[?age>30]" --format json
```

### Validate and Repair

```bash
datamorph validate data.json --schema schema.json
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

UPL navigates nested data structures:

| Syntax | Meaning |
|--------|---------|
| `.field` | Access object field |
| `[0]` | Index into array |
| `[*]` | Wildcard -- all elements |
| `[?age>30]` | Filter array by condition |
| `users[?active==true].name` | Chain filter + field |

## Supported Formats

| Format | Read | Write |
|--------|------|-------|
| JSON | Yes | Yes |
| YAML | Yes | Yes |
| TOML | Yes | Yes |
| CSV | Yes | Yes |

## Roadmap

- XML support
- SQLite output
- In-place editing with backups
- Batch processing with glob patterns
- Progress bars for large files

## License

MIT -- Copyright (c) 2026 Akshay
