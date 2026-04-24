# Datamorph — User Guide

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Commands](#commands)
   - [Convert](#convert)
   - [Query](#query)
   - [Validate](#validate)
   - [Repair](#repair)
   - [Diff](#diff)
   - [Lint](#lint)
4. [Universal Path Language (UPL)](#universal-path-language-upl)
5. [Format Support](#format-support)
6. [Tips & Tricks](#tips--tricks)
7. [Troubleshooting](#troubleshooting)

---

## Installation

See [README.md](../README.md#installation) for quick options.

**Quick (Linux/macOS):**
```bash
curl -fsSL https://raw.githubusercontent.com/aksaayyy/Datamorph/main/scripts/install.sh | bash
```

**Manual (download binary):**
```bash
# Choose your platform:
wget https://github.com/aksaayyy/Datamorph/releases/latest/download/datamorph-linux-amd64 -O ~/.local/bin/datamorph
chmod +x ~/.local/bin/datamorph
```

---

## Quick Start

```bash
# Convert JSON → YAML
datamorph convert data.json --to yaml -o data.yaml

# Query data (extract all names)
datamorph query users.json "users[*].name" --format json

# Diff two files
datamorph diff old.yaml new.yaml

# Validate JSON file (checks parse only)
datamorph validate config.json --schema schema.json

# Lint and auto-fix YAML files
datamorph lint *.yaml --fix
```

---

## Commands

### Convert

Convert data between formats.

**Syntax:**
```bash
datamorph convert [INPUT] [OUTPUT] --to <format> [OPTIONS]
```

**Arguments:**
- `INPUT` — source file path (`-` for stdin; omitted also reads stdin)
- `OUTPUT` — destination file path (`-` for stdout; omitted also writes stdout)

**Options:**
- `--from <format>` — explicitly set input format (json/yaml/toml/csv); auto-detected if omitted
- `--to <format>` — **required** — target format
- `--pretty` / `-p` — pretty-print output (indentation)
- `--in-place` / `-i` — overwrite input file with converted data (creates `.bak` backup)
- `--verify` — round-trip check: parse output to ensure it's valid

**Examples:**

```bash
# Read from stdin, write to stdout (JSON → YAML)
echo '{"a":1}' | datamorph convert --from json --to yaml -

# File to file with auto-detection
datamorph convert data.json --to toml -o config.toml

# In-place conversion (backup created)
datamorph convert old.csv --to json -i

# With verification
datamorph convert input.yaml --to json --verify
```

**Notes:**
- If `--from` is omitted, format is detected from the file extension or content
- Output encoding matches input (UTF-8); binary formats not supported

---

### Query

Extract, filter, or transform data using Universal Path Language (UPL).

**Syntax:**
```bash
datamorph query <input> <query> [OPTIONS]
```

**Arguments:**
- `<input>` — file path to query (required)
- `<query>` — UPL expression (required)

**Options:**
- `--format <fmt>` / `-f` — output format for results (default: json)

**Examples:**

```bash
# Get top-level field
datamorph query data.json ".name"

# Access array element by index
datamorph query data.json "users[0]"

# Wildcard: get all names from users array
datamorph query data.json "users[*].name"

# Filter: users where age > 30
datamorph query data.json "users[?age>30]"

# Chain: get emails of active users
datamorph query data.json "users[?active==true].email"

# Numeric comparison
datamorph query sales.json "orders[?total>1000].id"

# Nested wildcard
datamorph query org.json "departments[*].employees[*].name"

# Output as YAML
datamorph query data.json "items[*]" --format yaml
```

**Result types:**
- Single value → prints that value in output format
- Array → array in output format
- Multiple matches → always array (even single element)

---

### Validate

Check if a file is well-formed and optionally validate against a schema.

**Syntax:**
```bash
datamorph validate <input> --schema <schema_file>
```

**Options:**
- `--schema` — JSON Schema file (currently only JSON Schema supported)

**Examples:**

```bash
# Basic syntax check (any format)
datamorph validate config.yaml

# Validate against JSON Schema
datamorph validate data.json --schema schema.json
```

**Notes:**
- Currently only checks parseability; full JSON Schema validation planned
- Returns exit code 0 if valid, 1 if invalid

---

### Repair

Attempt to fix common formatting errors automatically.

**Syntax:**
```bash
datamorph repair <input> [--output <file>]
```

**Options:**
- `--output` / `-o` — write repaired output to file (default: overwrite input with `.bak` backup)

**Examples:**

```bash
# Auto-fix and overwrite with backup
datamorph repair corrupt.json

# Write to new file
datamorph repair broken.yaml -o fixed.yaml
```

**What gets repaired:**
- **JSON:** trailing commas, unbalanced braces/brackets, missing quotes on strings
- **YAML/TOML/CSV:** basic syntax errors are left to parser; currently only JSON has heuristics

**Limitations:**
- Repair is lossy; may discard data if input is severely malformed
- Always keep backups (`--in-place` creates `.bak`)

---

### Diff

Compare two data files structurally.

**Syntax:**
```bash
datamorph diff <file1> <file2>
```

**Arguments:**
- `file1`, `file2` — paths to compare (required)

**Examples:**

```bash
# Compare two JSON files
datamorph diff config_v1.json config_v2.json

# Compare YAML with different order
datamorph diff old.yaml new.yaml
```

**Output:**
```
✓ Files match — identical structure and values
✗ Files differ
  file1 — 142 bytes, 5 fields
  file2 — 138 bytes, 5 fields
```

**Notes:**
- Compares AST equality, not text
- Field order ignored (BTreeMap normalization)
- Whitespace and formatting ignored

---

### Lint

Check files for common issues and optionally fix them.

**Syntax:**
```bash
datamorph lint <files>... [--fix]
```

**Arguments:**
- `files` — one or more file paths (supports glob patterns with shell)

**Options:**
- `--fix` / `-f` — attempt to repair each file automatically

**Examples:**

```bash
# Lint a single file
datamorph lint config.yaml

# Lint multiple files
datamorph lint *.json *.yaml

# Lint and fix
datamorph lint data/*.csv --fix
```

**Output:**
```
✓ data.json  json
✗ broken.yaml — Parse error: ...
    → Fixed
```

---

## Universal Path Language (UPL)

UPL is Datamorph's query syntax for navigating nested data structures.

### Path Segments

| Syntax | Meaning | Example |
|--------|---------|---------|
| `.field` | Access object key | `.name` |
| `field` (leading) | Same as above | `users` |
| `[0]` | Array index (zero-based) | `users[0]` |
| `[*]` | Wildcard — all elements | `users[*]` |
| `[?cond]` | Filter by predicate | `users[?age>30]` |

### Predicates

Filter conditions use comparison operators:

```upl
field > number
field < number
field >= number
field <= number
field == "string"
field == true/false
```

**Examples:**

- `users[?active==true]` — only active users
- `items[?price>100]` — expensive items
- `orders[?status=="shipped"]` — shipped orders

### Chaining

Combine multiple segments:

```upl
users[?active==true].email        # get emails of active users
departments[*].employees[*].name  # all employee names in all departments
items[0].tags[*]                  # all tags of first item
```

### Operators

Binary operators (in expressions):

| Operator | Meaning |
|----------|---------|
| `+` `-` `*` `/` | Arithmetic (int only) |
| `==` `!=` | Equality / inequality |
| `>` `<` `>=` `<=` | Comparison |
| `&&` `||` | Logical AND/OR (planned) |

### Limitations

- Variables not supported yet (`$x`)
- No function calls (map, reduce) yet (filter implemented)
- Arithmetic only on integers (floats planned)
- No string operations (concatenation planned)

---

## Format Support

### JSON

**Capabilities:** Full read/write, pretty-printing, round-trip
**Limitations:** Comments discarded (JSON has none)

**Detected by:** First non-whitespace char is `{` or `[`

### YAML

**Capabilities:** Read/write, preserves structure (comments lost)
**Dependencies:** `serde_yaml` 0.9

**Detected by:** Content starts with `---` or `...` (YAML document markers), otherwise heuristic

**Notes:**
- YAML 1.2 supported
- Complex anchors/aliases may not round-trip perfectly

### TOML

**Capabilities:** Read/write, pretty-printed output
**Dependencies:** `toml` 0.8

**Detected by:** File extension `.toml` or starts with `[` (table)

**Notes:**
- TOML tables and arrays of tables fully supported
- Datetimes parsed as strings (no dedicated datetime type in AST yet)

### CSV

**Capabilities:** Read/write with automatic header detection and type inference
**Dependencies:** `csv` 1.4

**Parsing behavior:**
- Header row auto-detected via heuristics (unless `--no-header` flag added)
- Column types inferred per cell: int, float, bool, string
- Empty cells → empty strings
- Output CSV always includes header row

**Detected by:** File extension `.csv` or presence of commas in first lines

**Limitations:**
- Nested structures not supported (CSV is flat)
- Binary data not supported

---

## Tips & Tricks

### Pipeline workflows

```bash
# Convert on the fly
cat data.yaml | datamorph convert --from yaml --to json - > data.json

# Query CSV directly
datamorph query data.csv "rows[*].column_name" --format json

# Diff two YAML files after normalizing
datamorph diff file1.yaml file2.yaml
```

### Use with jq (for JSON only)

Datamorph is complementary to `jq`:

```bash
# Convert to JSON, then use jq for complex transforms
datamorph convert data.yaml --to json - | jq '.users | map(select(.age>30))'

# Or use jq first, then datamorph for format conversion
jq -f transform.jq input.json | datamorph convert --from json --to yaml -
```

### Batch processing

```bash
# Convert all YAML to JSON
for f in *.yaml; do
  datamorph convert "$f" --to json -o "${f%.yaml}.json"
done

# Or with parallel (GNU parallel)
parallel 'datamorph convert {} --to json -o {.}.json' ::: *.yaml
```

### In-place with backup

```bash
# Convert CSV to JSON, preserving original
datamorph convert data.csv --to json --in-place
# Creates data.csv.bak, replaces data.csv with JSON
```

### Debugging format detection

```bash
# Force a specific input format if auto-detection fails
datamorph convert weird_file --from json --to yaml
```

### Pretty output for humans

```bash
# Use pretty flag for formatted output
datamorph convert config.toml --to yaml --pretty -o pretty.yaml

# Disable colors if piping to file
datamorph --no-color query data.json "users" > output.txt
```

---

## Troubleshooting

### "Unsupported format" error

**Problem:** `error: unsupported format: xxx`

**Cause:** File extension not recognized or `--from` flag missing for unknown extension.

**Fix:** Use `--from` to specify format explicitly:
```bash
datamorph convert data.xyz --from json --to yaml
```

### CSV type inference wrong

**Problem:** Numbers parsed as strings or vice versa.

**Cause:** Heuristic type detection may misclassify edge cases (e.g., leading zeros).

**Workaround:** Ensure data is clean; pre-process with `sed` if needed. Future versions may allow explicit column type hints.

### Query returns empty array

**Problem:** `users[*].name` → `[]` when data exists.

**Cause:** Path doesn't match structure. Check data shape with `datamorph convert --pretty` first.

**Debug:** Use less-specific path to inspect:
```bash
datamorph query data.json "users"  # see what users actually contains
```

### "Parse error" on valid YAML

**Problem:** Valid YAML fails to parse.

**Cause:** YAML with advanced features (anchors, complex tags) may not be fully supported by `serde_yaml`.

**Workaround:** Simplify YAML or convert to JSON first.

### Binary not found after install

**Problem:** `command not found: datamorph`

**Cause:** `~/.local/bin` not in PATH.

**Fix:** Add to shell profile:
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission denied

**Problem:** Cannot execute binary.

**Cause:** Missing execute permission.

**Fix:** `chmod +x ~/.local/bin/datamorph`

---

## Performance Tips

- **Large CSV files (>100MB):** Consider using `--in-place` or streaming (future work). Currently loads entire file into memory.
- **Multiple queries:** Run once, save AST as intermediate JSON, query repeatedly.
- **Batch conversion:** Use shell loops or `parallel` for bulk operations.

---

## Getting Help

- **Report bugs:** https://github.com/aksaayyy/Datamorph/issues
- **Ask questions:** Use GitHub Discussions
- **Contributing:** See [CONTRIBUTING.md](../CONTRIBUTING.md)

