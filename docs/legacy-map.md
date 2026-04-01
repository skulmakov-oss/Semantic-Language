# Legacy Map (Final Stage)

## Status

Root is now **shim + bins only**.

Allowed in `root/src`:
- `src/lib.rs`
- `src/bin/smc.rs`
- `src/bin/svm.rs`
- `src/bin/ton618_core.rs`

Everything else was migrated to workspace crates, moved to assets, or removed.

## Root Final Structure

```text
src/
  lib.rs
  bin/
    smc.rs
    svm.rs
    ton618_core.rs
```

## What Was Removed from Root

- legacy backend source modules under `src/` (frontend/semantics/emit/vm adapters)
- shim side-files:
  - `src/frontend_shim.rs`
  - `src/semantics_shim.rs`
  - `src/semcode_format_shim.rs`
  - `src/semcode_vm_shim.rs`
- root sample data files moved to assets:
  - `src/human.sm` -> `assets/legacy_cli/human.sm`
  - `src/machine.sem` -> `assets/legacy_cli/machine.sem`
  - `src/profile.json` -> `assets/legacy_cli/profile.json`
  - `src/samples.json` -> `assets/legacy_cli/samples.json`

## Compatibility Layer

Compatibility re-exports remain in `src/lib.rs` as inline modules:
- `frontend`
- `semantics`
- `semcode_format`
- `semcode_vm`

No external shim files are used.

Remaining compatibility perimeter:

- `src/bin/ton618_core.rs`
  - retained as a legacy CLI shim for pre-v1 `ton618_core` workflows
  - not a canonical public CLI owner
- `crates/ton618-core`
  - retained as a compatibility-named low-level primitive crate
  - not a second owner for `sm-*` platform contracts

## Guards

`tests/legacy_guards.rs` enforces:
- no path adapters from crates to root (`#[path = "../../../src/..."]`)
- exact root/src allowlist policy (`lib.rs`, `smc.rs`, `svm.rs`, `ton618_core.rs`)
- ban of legacy patterns in root source (`legacy_`, `#[path =`, `include!`, `mod legacy`)
- explicit compatibility markers on the allowlisted `ton618` shim
- narrow allowlist for remaining `ton618` naming
- removal of `src/bin/support/` as a required invariant

