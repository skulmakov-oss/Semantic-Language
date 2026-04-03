# Persistence And Replay Backend Scope

Status: proposed post-stable expansion track
Related backlog item: `persistence and replay backends`

## Goal

Extend the current owner-split PROMETHEUS runtime baseline with explicit
persistence and replay backend artifacts without widening execution semantics,
host behavior, or the published `v1.1.1` contract by implication.

This is a post-stable runtime/storage expansion track, not a stable-line
correction.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `prom-state` owns in-memory semantic state, snapshots, restore, and state
  validation invariants
- `prom-audit` owns audit trail shape and replay metadata shape
- `prom-runtime` owns execution orchestration and session wiring only
- current runtime validation covers deterministic single-session execution, not
  persisted archives or replay backends
- persistence backends and multi-session replay archives are not part of the
  published `v1.1.1` commitment

That stable reading remains the source of truth until this track explicitly
lands a widened post-stable contract.

## Included In This Track

- explicit owner-layer for persistence/replay backend artifact families
- deterministic persisted envelope for `StateSnapshot` materialization and
  loading
- deterministic persisted envelope for `AuditTrail` / `ReplayMetadata`
  materialization and loading
- inspectable review formatting and metadata for persisted state/audit archives
- tests/docs/spec sync for the admitted persisted backend contract

## Explicit Non-Goals

- distributed or remote persistence backends
- database-specific integrations
- automatic recovery or crash-resume orchestration
- widening host ABI, `prom-*` capability policy, or rule-side effect semantics
- silent reopening of the published `v1.1.1` runtime boundary
- cross-version migration or rollback semantics beyond explicit persisted
  metadata
- CLI redesign or new packaged-service layout

## Intended Slice Order

1. docs/governance checkpoint
2. persisted backend artifact ownership and canonical envelope types
3. snapshot persistence materialization/loading for `prom-state`
4. audit/replay archive materialization/loading for `prom-audit`
5. docs/spec/runtime-validation freeze for the widened persisted contract

## Acceptance Reading

This track is done only when:

- persisted snapshot and replay artifacts have explicit owner crates
- persisted format metadata is deterministic and reviewable
- materialize/load behavior agrees with existing `prom-state` and `prom-audit`
  schema ownership
- runtime-facing docs distinguish the published `v1.1.1` baseline from the new
  post-stable persisted backend contract
- no part of the work quietly widens runtime recovery, host behavior, or rule
  execution semantics

## Slice History

1. docs/governance checkpoint
2. persisted backend artifact ownership and canonical envelope types in
   `prom-state` / `prom-audit`
