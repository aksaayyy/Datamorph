# Contributing to Datamorph

Thank you for your interest in contributing! This document provides guidelines and information for contributors.

## Getting started

### Prerequisites

- Rust toolchain (stable) — install via [rustup](https://rustup.rs)
- Git

### Development setup

```bash
git clone https://github.com/aksaayyy/Datamorph.git
cd Datamorph
cargo test        # run test suite
cargo build        # build debug
cargo build --release  # optimized build
```

### Running locally

```bash
./target/debug/datamorph --help
```

## Testing

We use Rust's built-in test framework. Please add tests for new features.

```bash
cargo test          # all tests
cargo test --release  # release mode tests
cargo test query::tests  # specific module
```

## Code style

- Follow Rust standard conventions (run `cargo fmt` to format)
- Use `clippy` for linting: `cargo clippy --all-targets -- -D warnings`
- Prefer explicit error handling with `Result` and `DataMorphError`
- Keep functions small and documented

## Submitting changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Commit your changes with clear messages
4. Push and open a Pull Request against `main`
5. Ensure CI passes (tests + formatting)

## Code of Conduct

Please adhere to the [Code of Conduct](CODE_OF_CONDUCT.md). Be respectful and constructive.

## Reporting issues

- Use GitHub Issues for bugs, feature requests, questions
- Provide steps to reproduce, expected vs actual behavior
- Include OS, Rust version (`rustc --version`), Datamorph version

## Questions?

Open an issue or reach out at akshayr3435@gmail.com
