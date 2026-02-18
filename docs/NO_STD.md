# no_std Support Matrix

This document defines the current `no_std` and `alloc` boundaries in the workspace-crate layout.

## Build Modes

- `std` (default): full toolchain (`exo-cli` + workspace compiler pipeline).
- `no_std` (`--no-default-features`): core/runtime-safe components only.
- `alloc` (crate-level): parser/semantics core logic without OS/file-system dependencies.

## Crate Matrix

- `crates/exo-core`: `no_std` + `alloc` native.
- `crates/exo-frontend`: `alloc`-capable; `std` optional.
- `crates/exo-semantics`:
  - `alloc_core` module: alloc-native semantic core.
  - `std_adapters` module: std-only glue (module loading, path/IO, diagnostic rendering context).
- `crates/exo-cli`: `std` only.

## Semantics Split (Contract)

- `alloc_core` contains:
  - type/symbol tables and semantic policies
  - import/export policy core checks
  - re-export/cycle/collision core helpers
  - pure law/when helpers and folding detectors
- `std_adapters` contains only:
  - provider/path orchestration for module loading
  - conversion of core policy errors into rendered diagnostics
  - CLI-facing std integration points

No file-system/path canonicalization is allowed inside `alloc_core`.

## Required Checks

Run before merge:

```powershell
cargo test -q
cargo check --no-default-features --quiet
cargo check -p exo-frontend --no-default-features --features alloc --quiet
cargo check -p exo-semantics --no-default-features --features alloc --quiet
```

## Scope

`no_std` is intended for embedding core compiler/runtime primitives into VectorOS contexts.
`exo-cli` and std adapters remain host-side orchestration layers.
