# Datamorph Architecture

## Overview

Datamorph is a **universal data format transformer** written in Rust. It converts between JSON, YAML, TOML, and CSV using a unified intermediate representation (AST) and a pluggable adapter system.

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Interface                          │
│  (clap — subcommands: convert, query, validate, repair,   │  │  diff, lint)                                                │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Command Dispatcher                        │
│  (src/main.rs — run_command)                                │
└─────────────┬────────────────────┬──────────┬────────────────┘
              │                    │          │
    ┌─────────▼──────┐    ┌────────▼────┐    │
    │   Convert      │    │   Query     │    │ (other commands)
    │ (format IO)    │    │ (UPL eval)  │    │
    └────────┬───────┘    └──────┬──────┘    │
             │                   │           │
             └──────────┬────────┘           │
                        │                    │
                        ▼                    ▼
        ┌──────────────────────────────────────────────┐
        │           Adapter Registry                    │
        │  (src/adapters/mod.rs — Adapter enum)         │
        │  JSON │ YAML │ TOML │ CSV                     │
        └────────┬──────────────┬──────────────┬───────┘
                 │              │              │
                 ▼              ▼              ▼
        ┌──────────────────────────────────────────────┐
        │         Format-specific Parsers               │
        │  serde_json | serde_yaml | toml | csv crate  │
        └───────────────┬──────────────┬───────────────┘
                        │              │
                        ▼              ▼
        ┌─────────────────────────────────────────────┐
        │           Canonical AST: Value              │
        │  (src/ast.rs — enum Value)                  │
        │  Null, Bool, Integer, Float, String,        │
        │  Array(Vec<Value>), Object(BTreeMap)        │
        └─────────────────────┬───────────────────────┘
                              │
             ┌────────────────┼────────────────┐
             │                │                │
             ▼                ▼                ▼
    ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
    │ Query Engine│  │ Diff Engine │  │ Lint/Repair │
    │ (UPL evalua-│  │ (structural │  │ (format-    │
    │   tor)      │  │  comparison)│  │   specific) │
    └─────────────┘  └─────────────┘  └─────────────┘
```

## Core Components

### 1. AST (Abstract Syntax Tree)

**File:** `src/ast.rs`

The `Value` enum is the universal representation of all data across formats. It's deliberately minimal:

- `Null` — JSON null / YAML null
- `Bool(bool)` — boolean
- `Integer(i64)` — whole numbers
- `Float(f64)` — decimal numbers
- `String(String)` — text
- `Array(Vec<Value>)` — ordered list
- `Object(BTreeMap<String, Value>)` — key-value map (sorted for deterministic output)

Derives: `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize` (for serde round-trips).

### 2. Format Adapters

**Files:**
- `src/adapters/mod.rs` — Adapter enum + registry functions
- No separate adapter files — specialized parsing done inline for performance

The `Adapter` enum encodes each format:

| Variant | Parse | Serialize |
|---------|-------|-----------|
| `Json`  | `serde_json::from_str` → `Value` | `serde_json::to_string_pretty` |
| `Yaml`  | `serde_yaml::from_str` | `serde_yaml::to_string` |
| `Toml`  | `toml::from_str` | `toml::to_string_pretty` |
| `Csv`   | `CsvAdapter::parse()` — uses `csv` crate with type inference | `CsvAdapter::serialize()` |

**Adapter methods:**
- `parse(&self, input: &str) -> Result<Value, DataMorphError>`
- `serialize(&self, value: &Value) -> Result<String, DataMorphError>`

### 3. CSV Adapter (`src/csv_adapter.rs`)

CSV is the only format that requires custom handling:

**Parsing:**
- Uses `csv::Reader` with flexible mode (heterogeneous columns)
- Probes first 10 lines to detect delimiter (`,`, `\t`, `;`, `|`)
- Heuristic to guess if first row is header (keyword matching, mixed case)
- **Type inference** per cell:
  - Empty → `String("")`
  - Integer regex → `Integer(i64)`
  - Float regex → `Float(f64)`
  - "true"/"false" → `Bool`
  - Otherwise → `String`
- Produces `Value::Array` of `Value::Object` (each row → map)

**Serialization:**
- Expects `Value::Array` of `Value::Object`
- Collects keys from first object as headers
- Converts each `Value` back to string via `value_to_string()`
- Uses `csv::Writer` with headers

### 4. Query Engine (UPL — Universal Path Language)

**File:** `src/query.rs`

UPL is a simple path language for navigating nested data:

**Grammar:**
```
path        := segment ('.' segment)*
segment     := field | index | wildcard | filter
field       := /[a-zA-Z_][a-zA-Z0-9_]*/
index       := '#' <integer>
wildcard    := '*'
filter      := '[' '?' condition ']'
condition   := field op value
op          := '==' | '!=' | '>' | '<' | '>=' | '<='
```

**AST:**
```rust
pub enum Expression {
    Path(Vec<PathSegment>),
    Filter { input: Box<Expression>, predicate: Box<Expression> },
    Map { input: Box<Expression>, transform: Box<Expression> },
    Literal(Value),
    Field(String),
    Variable(String),
    BinaryOp { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
}
```

**Evaluation:**
- `UplEvaluator::evaluate(expr, value) → Result<Value, QueryError>`
- Implements recursive descent through the expression tree
- Supports filtering arrays by predicate, mapping transformations, field access, array indexing, wildcards

**Operators supported (partial):**
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Arithmetic: `+`, `-`, `*`, `/` (only integers currently)
- Logical: `&&`, `||` (TODO)

**Examples:**
- `users[0].name` → Path: [Field("users"), Index(0), Field("name")]
- `users[?age>30].email` → Path with Filter + Field

### 5. Error Handling

**File:** `src/error.rs`

Custom error enum `DataMorphError` with `thiserror` and `miette` for rich diagnostics:

```rust
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    Io(#[from] io::Error),
    Parse(String),
    Format(String),
    Query(String),
    Repair(String),
    Diff(String),
    Lint(String),
    Validation(String),
    UnsupportedFormat(String),
    ConversionFailed(String),
    Multiple(Vec<Error>),
}
```

`From` conversions for all format library errors (serde_json, serde_yaml, toml, csv).

### 6. CLI

**File:** `src/main.rs`

- **Parser:** `clap` with `derive` for subcommands
- **Structure:** One function per command in `run_command()`
- **I/O:** `read_input()` handles stdin (`-`) or file paths; `write_output()` to stdout or file
- **Spinner:** `indicatif` progress bars for long operations (stub exists)
- **Color:** `colored` crate for terminal styling, disabled via `--no-color`

**Commands:**

| Command | Flags | Action |
|---------|-------|--------|
| `convert` | `--from`, `--to`, `--pretty`, `--in-place`, `--verify` | Parse → transform → serialize |
| `query` | `--format` | Parse → UPL evaluate → output |
| `validate` | `--schema` | Parse → check schema (stub) |
| `repair` | `--output` | Heuristic fixes per-format |
| `diff` | `file1 file2` | Structural diff, byte/field count |
| `lint` | `--fix` | Try parse, optionally repair |

## Data Flow: Convert Command

1. CLI parses args → `Commands::Convert { input, output, from, to, ... }`
2. `read_input()` loads data (file or stdin) → `String`
3. `detect_format()` if `--from` not given → extension or content sniffing
4. `Adapter::parse()` converts `String` → `Value` (AST)
5. `Adapter::serialize()` converts `Value` → output `String` in target format
6. `write_output()` writes to file or stdout

## Performance Considerations

- **Zero-copy where possible:** AST stores owned `String`/`Vec`, no boxing overhead
- **Streaming future work:** CSV could be streamed row-by-row (currently buffers full file)
- **Static linking (Linux musl):** Binaries are self-contained, no glibc dependencies
- **No heap allocation for small ints/floats:** Rust's enum layout is tight

## Extensibility

To add a new format (e.g., XML):
1. Add variant to `Adapter` enum
2. Implement parse/serialize logic in match arms
3. Add extension mapping in `extensions()`
4. Add detection logic (if auto-detect desired)
5. Update tests

## Testing

- **Unit tests:** `cargo test` — UPL parser tests in `query.rs`
- **Manual tests:** `test.sh` script (not in repo) covering all commands
- **No integration tests yet** — should add `tests/` directory with fixtures

## Dependencies (selected)

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` + format-specific | Deserialization/serialization |
| `csv` | CSV reading/writing with type inference |
| `miette` | Rich error messages with source spans |
| `thiserror` | Convenient Error enum derive |
| `colored` | Terminal colors |
| `indicatif` | Progress bars/spinners |
| `comfy-table` | Pretty tables (used in diff/lint) |
| `difflib` | Text diff algorithm |

## Future Architecture Changes

- **Streaming API:** `Iterator<Item = Value>` for CSV rows to reduce memory
- **Schema validation:** JSON Schema support via `jsonschema` crate
- **Parallel processing:** Rayon for multi-core query evaluation on large arrays
- **Plugins:** Dynamic loading of custom adapters via dlopen/LibLoader
- **LSP integration:** Language server for data files

---

Last updated: 2026-04-24
