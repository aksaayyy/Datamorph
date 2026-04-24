# Datamorph — Technical Design Document

**Version:** 0.1.0  
**Date:** 2026-04-24  
**Author:** Akshay  
**Language:** Rust 2021 edition

---

## Abstract

Datamorph is a command-line utility for transforming structured data between JSON, YAML, TOML, and CSV formats. It introduces a universal intermediate representation (AST) and a simple path-based query language (UPL) that works uniformly across all supported formats. The tool prioritizes ease of use, format interoperability, and cross-platform deployment via static compilation.

---

## 1. Problem Statement

Data engineers and developers frequently need to convert configuration files, export datasets, or inspect structured data stored in different formats. Existing tools are fragmented:

- `jq` — excellent for JSON but useless for YAML/TOML/CSV
- `yq` — YAML fork of jq, but CSV and TOML support limited
- `csvkit` — focused solely on CSV
- `toml-cli` — TOML-only

No single tool provides **universal** conversion, query, and manipulation across **all** common data formats with a consistent interface.

---

## 2. Design Goals

| Goal | Rationale |
|------|-----------|
| **One tool, all formats** | Reduce context switching; single mental model |
| **Zero config** | Auto-detection of formats; sensible defaults |
| **Fast** | Rust implementation; static linking; minimal allocations |
| **Portable** | Single binary per platform; no runtime dependencies |
| **Extensible** | New formats can be added without changing core |
| **Queryable** | Universal path language works across all formats |
| **Reliable** | Comprehensive error handling; round-trip guarantees |

---

## 3. Architecture Decisions

### 3.1 Intermediate AST vs Direct Transcoding

**Option A:** Direct transcoding (format A → format B via string manipulation)  
**Option B:** Universal AST (format A → AST → format B)

**Chosen: B — Universal AST**

**Why:**
- Single source of truth for data representation
- Query engine works on AST (same for all formats)
- Adding new format only requires adapter to/from AST
- Enables future features: validation, transformation, schema inference

**Trade-offs:**
- Slight performance overhead (parse + serialize vs direct)
- Memory: full AST must be held in RAM
- Loss of format-specific metadata (YAML comments, TOML ordering)

### 3.2 Adapter Pattern

Each format implements a `FormatAdapter` trait:

```rust
trait FormatAdapter {
    fn name(&self) -> &'static str;
    fn extensions(&self) -> &'static [&'static str];
    fn detect(&self, data: &[u8]) -> bool;
    fn parse(&self, data: &[u8]) -> Result<Value>;
    fn serialize(&self, value: &Value, pretty: bool) -> Result<Vec<u8>>;
}
```

Initial design planned trait objects (`Box<dyn FormatAdapter>`), but final implementation uses an `enum Adapter` for better performance and exhaustive matching (no dynamic dispatch).

**Why enum over trait objects:**
- Compile-time dispatch (no vtable overhead)
- Exhaustive `match` ensures all formats handled
- Simpler type system (no `dyn` in signatures)

**Format-specific details:**

| Format | Library | Notes |
|--------|---------|-------|
| JSON | `serde_json` | Full support; pretty-print via `to_string_pretty` |
| YAML | `serde_yaml` 0.9 | Supports 1.2 spec; round-trip mostly works |
| TOML | `toml` 0.8 | Round-trip; preserves table structure |
| CSV | `csv` 1.4 + custom logic | Requires sniffing, type inference, header detection |

### 3.3 CSV as Special Case

CSV is the only tabular (2D) format. Converting to/from nested AST requires flattening:

**CSV → AST:**
- Each row → `Value::Object` (map from column name → cell value)
- Type inferred per cell: int, float, bool, or string
- Whole file → `Value::Array` of row objects

**AST → CSV:**
- Expects `Value::Array` of homogeneous objects
- Keys from first object become headers
- Non-object array elements serialized as empty rows

**Type inference algorithm:**
```
if empty → String("")
else if parse i64 → Integer
else if parse f64 → Float
else if "true"/"false" → Bool
else → String
```

Limitation: no way to override inferred types; relies on clean data.

### 3.4 Universal Path Language (UPL)

UPL is a **restricted** path syntax inspired by `jq` but simplified:

Grammar:
```
path     := segment ('.' segment)*
segment  := field | index | wildcard | filter
field    := identifier
index    := '#' integer
wildcard := '*'
filter   := '[' '?' expr ']'
expr     := field op value
op       := '==' | '!=' | '>' | '<' | '>=' | '<='
```

**Constraints (compared to jq):**
- No function calls (`.`, `[]` only)
- Filters only support simple comparisons, not complex expressions
- No arithmetic in paths (except inside filters)
- No variable binding or piping (`|`) yet

**Design rationale:**
- Sufficient for 80% of data extraction use cases
- Easier to implement and parse than full jq
- Can be extended in v0.2.0

**Parsing algorithm:**
- Single-pass character scanner (no regex)
- Stack of `PathSegment` enums built during scan
- Converted to `Expression::Path` for evaluator

**Evaluation algorithm:**
- Recursive descent evaluator `UplEvaluator::evaluate()`
- Each segment applied in sequence to current value
- Wildcards and filters produce arrays (even single matches)
- `Value::Clone` used extensively (could optimize with references later)

### 3.5 Error Handling Strategy

Using `thiserror` + `miette` for user-friendly diagnostics:

```rust
#[derive(Error, Debug, Diagnostic)]
pub enum DataMorphError {
    #[error(transparent)] Io(#[from] io::Error),
    #[error("parse error: {0}")] Parse(String),
    #[error("format error: {0}")] Format(String),
    #[error("query error: {0}")] Query(String),
    // ... variant per command
}
```

**Philosophy:**
- Every public API returns `Result<T, DataMorphError>`
- Internal errors propagate up without context loss
- User-facing messages are concise but actionable
- `miette::Diagnostic` enables `--verbose` with source spans (future)

---

## 4. Implementation Highlights

### 4.1 Type Conversion Between Formats

Each format library uses its own value type:
- JSON: `serde_json::Value`
- YAML: `serde_yaml::Value`
- TOML: `toml::Value`
- CSV: custom (String → Value via inference)

**Conversion:** Manual `match` mapping between these and our `Value` enum. This avoids `serde` as common supertrait (would require `Serialize` + `Deserialize` on `Value` anyway). Since `Value` already derives `Serialize`/`Deserialize`, we could use `serde` for conversion but manual mapping gives more control (e.g., JSON float NaN handling).

**Current approach:**
```rust
fn value_to_serde_json(v: &Value) -> serde_json::Value { match v { ... } }
```

Inefficient (allocates intermediate), but simpler. Future optimization: derive `From` implementations.

### 4.2 Format Detection

**Auto-detection order:**
1. If user passes `--from`, use that.
2. Else, look at file extension (OS path split).
3. Else, sniff content bytes:
   - JSON: starts with `{` or `[` after whitespace
   - YAML: starts with `---` or `...`
   - TOML: starts with `[` (table) or contains `=`
   - CSV: contains commas and no JSON braces

Detection is **first-match** in registry order (currently JSON, YAML, TOML, CSV). CSV detection is heuristic (last to avoid false positives).

### 4.3 Performance

**Benchmarks (rough, on 1MB JSON file):**
| Operation | Time |
|-----------|------|
| JSON → YAML convert | ~120ms |
| Query (10k array) | ~50ms |
| CSV → JSON (10k rows) | ~200ms |

**Memory:**
- AST overhead: ~2× original data size (Rust enum discriminant + alloc)
- CSV type inference: all data held in memory (no streaming yet)

**Optimizations possible:**
- Streaming CSV row-by-row
- Zero-copy parsing via `serde_json::Deserializer::from_read` (implement `From` for borrowed data)
- `Arc<Value>` for shared subtrees in query results

**Current bottleneck:** CSV adapter reads entire file into string, then parses. Could stream directly from file.

### 4.4 Cross-Platform Builds

Using Rust's cross-compilation:

**Linux:** `x86_64-unknown-linux-musl` → static binary (~2MB)
**macOS:** `xarch64-apple-darwin` (Intel/ARM) → dynamically linked to system libs
**Windows:** `x86_64-pc-windows-msvc` → `.exe`

Build automation via GitHub Actions (see `.github/workflows/release.yml`).

---

## 5. Testing Strategy

### 5.1 Unit Tests

Currently only UPL parser tests:
```rust
#[cfg(test)]
mod tests {
    #[test] fn test_parse_simple_path() { ... }
    #[test] fn test_parse_array_index() { ... }
    #[test] fn test_parse_wildcard() { ... }
}
```

### 5.2 Integration Tests (TODO)

Planned `tests/` directory with:
- `integration_tests.rs` — full command invocations via `assert_cmd`
- Fixtures: `fixtures/` with sample JSON/YAML/TOML/CSV files
- Round-trip property tests: A → convert → B → convert back → A should equal

### 5.3 Property-Based Testing (Future)

Use `proptest` to generate random AST trees, then verify:
- Serialize → parse yields equivalent AST
- Query results are deterministic
- CSV ↔ JSON conversions preserve row count

---

## 6. Security Considerations

- **No code execution:** Datamorph does not evaluate arbitrary code (unlike `jq` with `@sh` etc.)
- **No shell expansion:** Input files treated as pure data
- **Safe defaults:** No temp files created; stdin/stdout used where possible
- **No network:** All operations local (no remote fetches)
- **Memory safety:** Rust guarantees no buffer overflows, use-after-free
- **Path traversal:** Input file paths are taken as-is; no sanitization needed (user responsible)

**Potential risks:**
- Large files cause OOM (future: streaming + limits)
- Maliciously crafted CSV could exhaust memory via many columns (guard: column limit config option)

---

## 7. Known Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No streaming CSV | Large files (>1GB) may OOM | Split files first |
| YAML comments lost | Round-trip not perfect | Accept; no plans to preserve |
| TOML datetime → string | Type info lost | Future: proper datetime type |
| No JSON Schema yet | `validate` only syntax checks | Use external validator |
| Windows requires MSVC toolchain | No MinGW support yet | Build on Windows or use WSL |
| Query only supports simple expressions | No arithmetic in paths | Use jq for complex transforms |

---

## 8. Future Work

### v0.2.0
- JSON Schema validation (using `jsonschema` crate)
- Streaming CSV support (`Iterator<Item = Value>`)
- Progress bars for large files
- `--in-place` with `--backup` flag

### v0.3.0
- Full arithmetic in UPL: `users[?age*2>50]`
- String operations: `contains`, `startsWith`, regex
- `map` and `reduce` functions
- Variables and piping: `users | map(.name)`

### v1.0.0
- XML support (via `serde-xml-rs`)
- SQLite output (write query results to DB)
- In-memory database mode for multiple operations
- Configuration file for defaults

---

## 9. Comparison with Related Work

| Tool | Formats | Query | Language | Statically linked |
|------|---------|-------|----------|-------------------|
| **Datamorph** | 4 | UPL (simple) | Rust | ✅ |
| `jq` | JSON only | jq (full) | C | ❌ (dynamic) |
| `yq` (Mike Farah) | YAML/JSON/XML/CSV | jq | Go | ✅ |
| `tomlq` | TOML/JSON | jq | Rust | ✅ |
| `csvkit` | CSV/SQL | SQL | Python | ❌ |

**Differentiators:**
- Single tool for all common data formats
- Rust → small, fast, secure binaries
- Simple query language accessible to non-experts
- Focus on conversion first, query second (not full transformation language)

---

## 10. Conclusion

Datamorph fills a niche for developers who regularly switch between JSON, YAML, TOML, and CSV and need a reliable, fast, zero-dependency converter with basic query capabilities. Its architecture prioritizes simplicity and extensibility over being a full query language; it's not meant to replace `jq` for complex JSON transformations but to be the **first tool you reach for** when you need to "convert this YAML to JSON and grab a field."

Future work will close feature gaps (schema validation, streaming) while maintaining the philosophy of "one binary, zero dependencies, works everywhere."

---

## Appendix A: Build Instructions

```bash
# Clone
git clone https://github.com/aksaayyy/Datamorph.git
cd Datamorph

# Install musl target for static Linux binary
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl
# Output: target/x86_64-unknown-linux-musl/release/datamorph

# Package
mkdir -p release
cp target/x86_64-unknown-linux-musl/release/datamorph release/datamorph-linux-amd64
strip release/datamorph-linux-amd64  # optional
```

**Cross-compilation for macOS/Windows:** Use GitHub Actions or build on respective OS.

---

## Appendix B: Release Checklist

- [ ] Bump version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Build all platform binaries (Linux static, macOS Intel/ARM, Windows)
- [ ] Test each binary manually
- [ ] Create GitHub release with assets
- [ ] Update Homebrew formula (if maintained)
- [ ] Announce on relevant channels

---

*End of document*
