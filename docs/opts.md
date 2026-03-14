# Optimization Passes (v0.2)

## StructuralCleanup

- Location: `crates/sm-ir/src/passes/cleanup.rs`
- Type: IR optimization pass (`OptPass`)
- Entry: `run_default_opt_passes()` in `crates/sm-ir/src/passes/mod.rs`

StructuralCleanup is an IR-stage cleanup pass. It is not part of lowering itself, and its
implementation must not live in `legacy_lowering.rs`.

## CrystalFold

- Location: `crates/sm-ir/src/passes/crystalfold.rs`
- Type: IR optimization pass (`OptPass`)
- Entry: `run_default_opt_passes()` in `crates/sm-ir/src/passes/mod.rs`

CrystalFold is an IR-stage pass. It is not part of parsing, smc typing, or emit.

## W0241 Scope

- `W0241` is emitted by semantics as a hint/diagnostic only.
- `W0241` does not guarantee a rewrite happened.
- Materialized rewrites are performed by CrystalFold on IR.

## Guarantees

- Deterministic: linear instruction traversal, stable rewrite order.
- Idempotent: applying CrystalFold twice is equivalent to once.
- Covered by test: `crystalfold_idempotent`.

## Pipeline Order

`frontend -> semantics (warnings) -> lowering -> StructuralCleanup -> CrystalFold -> emit`

