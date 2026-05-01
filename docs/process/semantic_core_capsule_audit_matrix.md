# Semantic Core Capsule Audit Matrix

Status: audit snapshot as of 2026-05-01

Scope:

- this is a status audit of the current main-workspace implementation
- this is not a reconstruction of historical PR sequence
- the matrix below evaluates the built code and current acceptance evidence

## Summary

- Closed waves: `19 / 21`
- Partial waves: `2 / 21`
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
| `CORE-00` | Partial | workspace members exist; sealed capsule facade exists; `cargo doc -p semantic-core-capsule --no-deps` builds | `BackendKind` still appears in capsule docs through public fields on `CoreConfig` and `CoreResult` |
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
| `CORE-20` | Partial | execution docs exist and public CLI output is clean | forbidden words still exist in public-core test sources, so strict grep-based wording hygiene is not fully closed |

## Package-Level Gaps

### `CORE-00B`

Observed gap:

- `CoreConfig` and `CoreResult` still expose `backend: BackendKind` in the public capsule-facing docs path

Impact:

- the facade no longer exports backend methods, but backend naming still leaks through field types
- this makes `CORE-00B` only partially closed under the stricter reading of “backend detail does not appear in crate docs”

Likely fix options:

- make those fields private and expose accessor methods through a narrower capsule contract
- or move backend selection and reporting fully outside capsule-facing result/config types

### `CORE-20B`

Observed gap:

- forbidden words are still present in public-core test sources:
  - `crates/semantic-core-exec/src/lib.rs`
  - `crates/core-lab/tests/help_hygiene.rs`

Impact:

- shipped help text and docs are clean
- strict acceptance text of “grep forbidden words in public core returns empty” is not satisfied

Important note:

- `CORE-19B` and `CORE-20B` are in tension if implemented literally
- `CORE-19B` wants deny-list tests with explicit forbidden tokens
- `CORE-20B` wants those same tokens absent from the public-core tree

Likely fix options:

- narrow `CORE-20B` scope to shipped surfaces, docs, and comments
- or keep strict scope and move deny-list tokens out of literal source text in tests

## Recommended Next Actions

1. Resolve the `CORE-00B` boundary leak by removing `BackendKind` from capsule-facing public fields.
2. Decide whether `CORE-20B` should apply to shipped surfaces only or to the entire public-core tree including tests.
3. After that policy call, run a final wording-hygiene pass and freeze the matrix again.
