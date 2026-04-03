# PROMETHEUS Runtime Validation Policy

Status: draft v0

This document defines the current release-facing validation and regeneration discipline for the `prom-*` runtime baselines.

## Covered Surfaces

Current runtime validation artifacts:

- `tests/prometheus_runtime_matrix.rs`
- `tests/prometheus_runtime_goldens.rs`
- `tests/prometheus_runtime_negative_goldens.rs`
- `tests/prometheus_runtime_compat_matrix.rs`

Current snapshot directory:

- `tests/golden_snapshots/runtime/`

## Regeneration Rule

Runtime snapshots may be regenerated only when:

1. the public runtime contract changed intentionally
2. the relevant `docs/spec/*` file was updated
3. the resulting snapshot diff is reviewed as a contract change

Current regeneration switch:

- set `SM_UPDATE_SNAPSHOTS=1` only for intentional contract updates

Do not regenerate snapshots to hide an unexplained behavioral diff.

## Review Rule

Before updating a runtime baseline, verify:

1. whether the change affects:
   - capability denial behavior
   - gate validation behavior
   - runtime session descriptor fields
   - audit event ordering
   - state-to-agenda orchestration flow
2. whether `docs/spec/runtime.md`, `docs/spec/audit.md`, `docs/spec/capabilities.md`, or `docs/spec/gates.md` need updates
3. whether the compatibility matrix still reflects the supported contract

## Known Limits

Current runtime baselines intentionally cover:

- deterministic gate-backed execution
- capability denial before host dispatch
- read-only gate write denial
- state validation rejection before orchestration progress
- canonical session descriptor exposure
- canonical persisted archive materialization/loading for:
  - `StateSnapshotArchive`
  - `AuditReplayArchive`
- canonical multi-session replay archive materialization/loading for:
  - `MultiSessionReplayArchive`
- canonical declared-order rule-side effect execution for:
  - `RuleEffect::StateWrite`
  - `RuleEffect::AuditNote`
- canonical rollback artifact ownership and deterministic apply/restore for:
  - `StateRollbackArtifact`
  - `SemanticStateStore::apply_rollback(...)`

Current runtime baselines do not yet cover:

- inter-session state migration
- mixed-family generic rule-effect executors
- rollback, retry, or compensation semantics for rule effects
- rollback artifact canonical text materialization/loading
- crash-resume, recovery, or repair semantics around rollback persistence

## Merge Gate

Changes touching the `prom-*` runtime contract should not merge unless:

- public API inventory guard passes for contract-sensitive crates
- CI runs the runtime validation quartet as explicit release-facing jobs
- runtime matrix tests pass
- runtime golden tests pass
- runtime negative golden tests pass
- runtime compatibility matrix test passes
