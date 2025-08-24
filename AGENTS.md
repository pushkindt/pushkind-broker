# AGENTS.md

This file provides guidance to AI code generators when working with the code in this repository.

## Development Commands

Use these commands to verify your changes before committing:

**Build**
```bash
cargo build --verbose
```

**Run Tests**
```bash
cargo test --verbose
```

**Lint (Clippy)**
```bash
cargo clippy --tests -- -Dwarnings
```

**Format**
```bash
cargo fmt -- --check
```

## Code Architecture

This is a Rust ZeroMQ application to handle multiple ZeroMQ proxies.

### Key Development Rules

- Use idiomatic Rust everywhere
- Use `thiserror` for error definitions; avoid `anyhow::Result`
- Define error types inside their unit of fallibility
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before all commits
- Document all public APIs and breaking changes
- Always run formatting and linting before create PRs
