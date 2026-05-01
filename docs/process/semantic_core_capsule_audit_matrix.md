# Semantic Core Capsule Audit Matrix

Status: audit snapshot as of 2026-05-01

Scope:

- this is a status audit of the current main-workspace implementation
- this is not a reconstruction of historical PR sequence
- the matrix below evaluates the built code and current acceptance evidence

## Summary

- Closed waves: `21 / 21`
- Partial waves: `0 / 21`
- Open functional execution gaps: none found in the current public core path
- Remaining gaps are boundary and wording-hygiene strictness issues

## Audit Commands

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo check -p semantic-core-quad --no-default-features`
- `cargo doc -p semantic-core-capsule --no-deps`
- `cargo run -p core-lab -- --help`
- `cargo run -p core-lab -- caps`
- `cargo run -p semantic-core-bench -- quad-reg`
- `cargo run -p semantic-core-bench -- tile`
- `cargo run -p semantic-core-bench -- exec`
- public-core wording search over `crates/semantic-core-*`, `crates/core-lab`, and `docs/core`

## Wave Matrix

| Wave | Status | Evidence | Notes |
| --- | --- | --- | --- |
| `CORE-00` | Closed | workspace members exist; sealed capsule facade exists; `cargo doc -p semantic-core-capsule --no-deps` builds without `BackendKind` in capsule docs; `CoreEnginePolicy` is the capsule-facing policy vocabulary | `CORE-00B` resolved: `BackendKind` removed from `CoreConfig`/`CoreResult` public surface; replaced by `CoreEnginePolicy` enum with `DeterministicReference` and `Auto` variants |
| `CORE-01` | Closed | `QuadState` exists with frozen encoding and exhaustive truth-table tests | no acceptance gap found |
| `CORE-02` | Closed | `QuadroReg32` exists with raw, lane, packed-op, and debug coverage | no acceptance gap found |
| `CORE-03` | Closed | `QuadMask32`, `QuadMasks32`, and register mask mutation APIs exist with tests | no acceptance gap found |
| `CORE-04` | Closed | `QuadTile128`, `QuadMask128`, and reg-to-tile conversion exist with tests | no acceptance gap found |
| `CORE-05` | Closed | `StateDelta32` and `StateDelta128` exist with transition coverage | no acceptance gap found |
| `CORE-06` | Closed | `QuadroBank` and `QuadTileBank` exist and per-bank ops are tested | no acceptance gap found |
| `CORE-07` | Closed | `CoreValue`, `Fx`, and checked fixed-point arithmetic exist with tests | no acceptance gap found |
| `CORE-08` | Closed | `CoreOpcode` and typed `Instr` exist; `Instr` size is compile-time frozen at 12 bytes | no acceptance gap found |
| `CORE-09` | Closed | `RegId`, `Frame`, `CoreFunction`, and `CoreProgram` exist with validation tests | no acceptance gap found |
| `CORE-10` | Closed | `CoreTrap` and `FuelMeter` exist with stable trap-code tests | root-frame `Ret` is intentionally normal completion, not a trap |
| `CORE-11` | Closed | scalar executor, branching, arithmetic, trap, assert, `call`, and `ret` are covered by tests and goldens | no acceptance gap found |
| `CORE-12` | Closed | `BackendKind`, `BackendCaps`, and internal `pub(crate)` backend trait exist | public-boundary leak is tracked under `CORE-00`, not here |
| `CORE-13` | Closed | scalar backend plus x86 and arm feature-detection scaffolds exist | no acceptance gap found |
| `CORE-14` | Closed | `CoreAdmissionProfile` and `validate_program` exist with structural checks | no acceptance gap found |
| `CORE-15` | Closed | SemCode bridge boundary exists as a stub loader and source trait; internal builder exists for tests | no acceptance gap found |
| `CORE-16` | Closed | golden programs exist; result digest exists; `.core.json` envelope is versioned with `format_version` | no acceptance gap found |
| `CORE-17` | Closed | seeded quad differential tests and bank tail-length tests exist | uses seeded tests rather than `proptest`, but acceptance is satisfied |
| `CORE-18` | Closed | `semantic-core-bench` runs `quad-reg`, `tile`, `exec`, and reports deterministic metric keys | no acceptance gap found |
| `CORE-19` | Closed | `core-lab` supports `run`, `validate`, `caps`, `bench`, and hygiene tests pass | no acceptance gap found |
| `CORE-20` | Closed | execution docs exist and public CLI output is clean; `CORE-20B` policy scoped to shipped surfaces | deny-list tokens may appear in test oracle files — that is required for the deny-list tests to function; scope is rustdoc, user-facing docs, CLI help/output, and public examples |

## Package-Level Gaps

### `CORE-00B` — Closed

Resolution:

- `backend: BackendKind` field removed from `CoreConfig` and `CoreResult`; `BackendKind` no longer appears anywhere in `semantic-core-capsule` docs
- new public type `CoreEnginePolicy { DeterministicReference, Auto }` defined in `semantic-core-exec` and re-exported from the capsule facade
- capsule-facing config API: `CoreConfig::engine_policy() -> CoreEnginePolicy`, `CoreConfig::with_engine_policy(CoreEnginePolicy) -> CoreConfig`
- `BackendKind` is strictly internal to `semantic-core-backend` and `semantic-core-exec`; not reachable from the capsule public API

### `CORE-20B` — Closed

Policy decision:

CORE-20B applies to **shipped public surfaces only**:
- rustdoc
- user-facing docs
- CLI help and output
- public examples

It does **not** apply to internal test oracle files. Deny-list tests must contain or construct the denied vocabulary in order to function; requiring them to be absent from test source would make the tests fight themselves.

Resolution:

- shipped surfaces are already clean (confirmed by audit commands)
- deny-list literals in `crates/semantic-core-exec/src/lib.rs` test section and `crates/core-lab/tests/help_hygiene.rs` are intentional oracle strings, not hygiene violations under the adopted policy

## Status

All 21 waves closed. Matrix frozen as of 2026-05-01.
