# AGENTS.md

## Project

serde-kdl2 is a Serde integration crate for KDL v2 (KDL Document Language).
It provides `Serialize` and `Deserialize` implementations that map Rust types
to and from KDL documents, built on the `kdl` crate v6.5.

## Repository Layout

- `src/lib.rs` — public API, re-exports, module-level docs
- `src/ser.rs` — serializer: Rust types → KdlDocument
- `src/de.rs` — deserializer: KdlDocument → Rust types
- `src/error.rs` — error enum with `thiserror` derives
- `tests/integration.rs` — integration tests covering mapping rules
- `tests/property.rs` — proptest-based property tests
- `justfile` — task runner (`just all` runs fmt, clippy, coverage)

## Build and Test

Requires Rust nightly (edition 2024).

```sh
just all        # fmt + clippy + coverage (enforces 100%)
just clippy     # clippy with -D warnings
just coverage   # instrumented test run, grcov report, fails below 100%
cargo test      # plain test run without coverage
```

## Conventions

- Rust edition 2024 (nightly toolchain)
- 100% line coverage required; CI fails otherwise
- `cargo clippy -- -D warnings` must pass with no warnings
- `cargo fmt` enforced in CI
- Use `thiserror` for error types
- Public items have doc comments
- Integration tests live in `tests/`, not inline

## CI

GitHub Actions runs on push to main and on PRs:

1. `cargo fmt --check`
2. `just clippy coverage`

Release workflow tags calver versions on successful CI runs against main.
Crates.io publishing is configured but currently commented out.

## Mapping Rules

The serializer and deserializer follow these conventions for converting between
Rust types and KDL nodes:

- Struct fields become nodes; the field name is the node name, the value is the
  first argument.
- Nested structs use children blocks.
- Vec of primitives becomes multiple arguments on one node.
- Vec of structs becomes repeated nodes with the same name.
- The `-` (dash) children convention is supported for deserialization.
- `Option::None` omits the node; `#null` also deserializes as `None`.
- Unit enum variants serialize as strings; newtype, tuple, and struct variants
  use child nodes named after the variant.
- Maps serialize identically to structs.

These rules are documented in `README.md` and `src/lib.rs`. Keep all three
locations consistent when changing mapping behavior.

## Version Control

Use jj (Jujutsu), not git.
