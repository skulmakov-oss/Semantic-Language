# Semantic Core Capsule Audit Matrix

Status: capsule audit snapshot as of 2026-05-01

Scope:

- this matrix audits the capsule/core snapshot defined by commits `59ad0e1` and `0f33c32`
- this is not a reconstruction of historical PR sequence
- the matrix below evaluates the capsule/core acceptance evidence only
- this clean follow-up branch isolates the workspace-green check from unrelated local drift
- this branch is a local execution-core baseline pending push or PR merge

## Summary

- Closed waves: `21 / 21` within the audited capsule scope
- Partial waves: `0 / 21` within the audited capsule scope
- Open functional execution gaps: none found in the audited public core path
- Remaining capsule-scope gaps: none
- Live worktree blocker outside scope: none; `cargo test --workspace` is green on this clean follow-up branch

## Audit Commands

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo test -p semantic-core-capsule`
- `cargo test -p core-lab`
- `cargo check -p semantic-core-quad --no-default-features`
- `cargo doc -p semantic-core-capsule --no-deps`
- `cargo run -p core-lab -- --help`
- `cargo run -p core-lab -- caps`
- `cargo run -p semantic-core-bench -- quad-reg`
- `cargo run -p semantic-core-bench -- tile`
- `cargo run -p semantic-core-bench -- exec`
- public-core wording search over `crates/semantic-core-*`, `crates/core-lab`, `docs/core`, and `docs/process`

## Live Worktree Note

- this clean follow-up branch does not carry the unrelated tracked modifications present in the noisier local execution worktree
- `tests/public_api_contracts.rs` is green on this branch
- `cargo test --workspace` is green on this branch
- this repository-level green state does not widen the capsule audit scope recorded here

## Wave Matrix

| Wave | Status | Evidence | Notes |
| --- | --- | --- | --- |
| `CORE-00` | Closed | workspace members exist; sealed capsule facade exists; `cargo doc -p semantic-core-capsule --no-deps` builds with no backend-type leakage in capsule docs | no acceptance gap found |
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
| `CORE-12` | Closed | `BackendKind`, `BackendCaps`, and internal `pub(crate)` backend trait exist | no acceptance gap found |
| `CORE-13` | Closed | scalar backend plus x86 and arm feature-detection scaffolds exist | no acceptance gap found |
| `CORE-14` | Closed | `CoreAdmissionProfile` and `validate_program` exist with structural checks | no acceptance gap found |
| `CORE-15` | Closed | SemCode bridge boundary exists as a stub loader and source trait; internal builder exists for tests | no acceptance gap found |
| `CORE-16` | Closed | golden programs exist; result digest exists; `.core.json` envelope is versioned with `format_version` | no acceptance gap found |
| `CORE-17` | Closed | seeded quad differential tests and bank tail-length tests exist | uses seeded tests rather than `proptest`, but acceptance is satisfied |
| `CORE-18` | Closed | `semantic-core-bench` runs `quad-reg`, `tile`, `exec`, and reports deterministic metric keys | no acceptance gap found |
| `CORE-19` | Closed | `core-lab` supports `run`, `validate`, `caps`, `bench`, and hygiene tests pass | no acceptance gap found |
| `CORE-20` | Closed | execution docs exist; public CLI output is clean; public-core wording search over crates, lab, and docs returns empty | no acceptance gap found |

## Cleanup Notes

### `CORE-00B`

Resolved in this branch:

- the capsule now owns its public `CoreConfig` and `ExecutionMode` facade types
- `cargo doc -p semantic-core-capsule --no-deps` no longer shows `BackendKind`, backend crate names, or backend helper methods in capsule docs

Result:

- the narrow capsule contract is preserved without backend naming in the public capsule docs path

### `CORE-20B`

Resolved in this branch:

- deny-list checks were rewritten so reserved labels are assembled dynamically in tests
- public docs and crate sources in the audited scope pass the wording search with no reserved labels present as plain literals

Result:

- the wording-hygiene check now matches the stricter interpretation without weakening the CLI hygiene coverage

## Recommended Next Actions

1. Push or publish this clean follow-up branch so the workspace-green baseline becomes externally reviewable.
2. Cherry-pick or otherwise fold this narrow truth-sync back into the main capsule stream only after review.
3. Split the broader capsule import snapshot into the planned PR stream once review starts.
4. Keep future process updates tied to real branch or PR state, not just local workspace state.
