# AGENTS.md

This document provides guidance to AI code generators when working in this
repository. Follow these practices so that new code matches the established
architecture and conventions.

## Project Context

`pushkind-broker` is a Rust 2024 binary crate that runs a minimal ZeroMQ broker
for Pushkind services. Configuration is loaded from `proxy.yaml`,
`proxy.toml`, or `proxy.json` by `src/config.rs`, validated, and turned into a
set of `Pair` definitions. `src/main.rs` initializes logging, loads settings,
and spawns a thread per pair, while `src/proxy.rs` owns the ZeroMQ socket setup
and call into `zmq::proxy`. All runtime logging goes through the `log` /
`env_logger` stack.

## Development Commands

Use these commands to verify your changes before committing:

**Full Check**
```bash
make check
```

**Build**
```bash
cargo build --all-targets --verbose
```

**Run Tests**
```bash
cargo test --all-targets --verbose
```

**Lint (Clippy)**
```bash
cargo clippy --all-features --tests -- -Dwarnings
```

**Format**
```bash
cargo fmt --all -- --check
```

## Coding Standards

- Write idiomatic Rust and propagate errors with `Result<_, BrokerError>`-style
  returns instead of panicking.
- Keep module responsibilities clear: configuration parsing and validation live
  in `src/config.rs`, ZeroMQ orchestration in `src/proxy.rs`, and application
  startup in `src/main.rs`.
- Use `thiserror` for error types, implementing `From` conversions for external
  errors so callers can bubble failures up cleanly.
- Avoid `unwrap`/`expect` outside of tests; log context and return meaningful
  errors.
- Pass configuration data or handles explicitly rather than introducing global
  mutable state.
- Document any new public APIs or breaking behavioral changes.

## Configuration Guidelines

- Maintain the `config::Config` loader order so YAML, TOML, and JSON files all
  work (`proxy.yaml` first, then `proxy.toml`, then `proxy.json`).
- Provide sensible defaults for new fields via `#[serde(default = "fn")]` or
  equivalent helpers (e.g., `default_hwm`).
- Validate incoming data and return `ConfigError::Invalid` when definitions are
  missing, empty, or inconsistent.
- Update `README.md` (and sample configs if present) when adding or renaming
  configuration keys.

## ZeroMQ Proxy Guidelines

- Configure socket options (e.g., high-water marks) before binding endpoints to
  avoid transient message loss.
- Keep `run_pair` focused on wiring sockets and delegating to `zmq::proxy`,
  returning `ProxyError::Zmq` for failures.
- Ensure sockets are placed into a zero-linger state before shutdown so threads
  exit promptly.
- Log pair labels and key lifecycle events to aid operational visibility.

## Testing Expectations

- Add unit tests alongside the modules they exercise using the existing
  `#[cfg(test)] mod tests` pattern.
- Use `tempfile` (already in dev-dependencies) for filesystem-backed config
  fixtures instead of hard-coding paths.
- Cover both success paths and invalid configurations or endpoints to keep error
  handling robust.
- Ensure new functionality is covered by tests before opening a pull request.

By following these principles the generated code will align with the project’s
architecture, technology stack, and long-term maintainability goals.
