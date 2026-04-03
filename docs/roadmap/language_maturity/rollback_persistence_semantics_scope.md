# Rollback Persistence Semantics Scope

Status: proposed post-stable expansion track
Related backlog item: `rollback persistence semantics`

## Goal

Extend the current persisted state/archive baseline with explicit rollback
artifact ownership and deterministic apply/review semantics without silently
widening the published `v1.1.1` contract, recovery behavior, or inter-session
migration guarantees.

This is a post-stable persistence/runtime expansion track, not a stable-line
correction.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `prom-state` owns canonical semantic state, snapshots, and
  `StateSnapshotArchive`
- current `main` admits deterministic persisted snapshot materialization/loading
- current `main` admits deterministic multi-session replay archive ownership and
  materialization/loading in `prom-audit`
- rollback persistence semantics are not part of the published `v1.1.1`
  commitment
- runtime validation still treats rollback persistence as outside the admitted
  baseline

That reading remains the source of truth until this track explicitly lands a
widened post-stable contract.

## Included In This Track

- explicit owner-layer rollback artifact family tied to canonical persisted
  state history
- deterministic rollback metadata and review formatting
- deterministic rollback apply/restore semantics for the admitted first-wave
  path
- docs/spec/runtime-validation sync for the widened rollback contract

## Explicit Non-Goals

- crash-resume or automatic recovery orchestration
- inter-session migration or repair semantics
- distributed persistence backends
- widening host ABI, capability policy, or rule-side effect execution
- silent reopening of the published `v1.1.1` stable boundary
- generic transaction engine or user-defined rollback families

## Intended Slice Order

1. docs/governance checkpoint
2. explicit rollback artifact ownership and metadata
3. deterministic rollback apply/restore path for the admitted first-wave
4. runtime/spec/validation freeze for the widened rollback contract

## Acceptance Reading

This track is done only when:

- rollback artifacts have explicit owner crates
- rollback metadata and review formatting are deterministic
- rollback apply/restore behavior agrees with existing state/archive ownership
- release-facing docs distinguish the published `v1.1.1` baseline from the new
  post-stable widened rollback contract
- no part of the work quietly widens recovery, migration, or generic
  transaction semantics

