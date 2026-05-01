# Multi-Session Replay Archive Scope

Status: completed post-stable first-wave track on `main`
Related backlog item: `multi-session replay archives`

## Goal

Extend the current persisted audit/state archive baseline from single-session
materialization into deterministic multi-session replay archive ownership and
reviewable loading rules without silently widening the published `v1.1.1`
contract, rollback semantics, or recovery behavior.

This is a post-stable persistence/runtime expansion track, not a stable-line
correction.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `prom-state` owns single-session `StateSnapshotArchive`
- `prom-audit` owns single-session `AuditReplayArchive`
- current `main` admits canonical text materialization/loading for those
  single-session archives only
- multi-session replay archives are not part of the published `v1.1.1`
  commitment
- runtime validation still treats multi-session replay archives as outside the
  admitted baseline

That reading remains the source of truth for the published `v1.1.1` tag.
Current `main` now carries a widened first-wave contract for deterministic
multi-session replay archive ownership and canonical materialization/loading
only.

## Included In This Track

- explicit owner-layer artifact family for deterministic multi-session replay
  archives
- canonical session-boundary metadata and ordering rules
- deterministic materialize/load behavior for the same archive family
- docs/spec/runtime-validation sync for the widened persisted replay contract

## Explicit Non-Goals

- rollback or compensation semantics
- inter-session migration or state repair
- crash-resume orchestration or automatic recovery
- remote/distributed persistence backends
- widening host ABI, capability policy, or rule-side effect execution
- silent reopening of the published `v1.1.1` stable boundary

## Intended Slice Order

1. docs/governance checkpoint
2. explicit owner-layer multi-session replay archive types
3. canonical materialize/load path for ordered session bundles
4. runtime/spec/validation freeze for the widened replay contract

## Slice History

- `f98ca1d` documented this track as a post-stable persistence/runtime
  expansion checkpoint
- `87a009b` added explicit `prom-audit` ownership for ordered
  `MultiSessionReplayArchiveSession` and `MultiSessionReplayArchive` types
- `b963f85` admitted canonical deterministic text
  materialization/loading for ordered multi-session replay bundles, while still
  excluding rollback/recovery semantics

## Completed First-Wave Reading

Current `main` now treats this track as completed for the narrow first-wave
contract:

- explicit owner-layer multi-session replay bundle types are canonical in
  `prom-audit`
- canonical deterministic text materialization/loading exists for ordered
  `MultiSessionReplayArchive` bundles
- embedded session archives remain ordered by monotonic `session_ordinal`
- published `v1.1.1` remains narrower and does not retroactively claim this
  widened persisted replay surface
- rollback, migration, recovery, and runtime replay engines remain outside this
  first-wave contract

## Acceptance Reading

This track is done only when:

- multi-session replay archives have explicit owner crates
- session ordering and metadata are deterministic and reviewable
- materialize/load behavior agrees with existing single-session archive owners
- release-facing docs distinguish the published `v1.1.1` baseline from the new
  post-stable widened replay contract
- no part of the work quietly widens rollback, migration, or recovery
  semantics

That acceptance boundary is now met on current `main`.
