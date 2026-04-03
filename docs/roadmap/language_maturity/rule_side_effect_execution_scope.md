# Rule Side-Effect Execution Scope

Status: proposed post-stable expansion track
Related backlog item: `richer rule-side effect execution semantics`

## Goal

Extend the current owner-split PROMETHEUS runtime baseline from deterministic
agenda activation into explicit rule-side effect execution semantics without
silently widening the published `v1.1.1` runtime contract.

This is a post-stable semantic-runtime expansion track, not a stable-line
correction.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- `prom-rules` owns rule identity, conditions, validation, and deterministic
  agenda ordering
- `prom-runtime` owns orchestration glue around activation selection, state
  update application, and audit emission helpers
- current runtime behavior covers rule activation and orchestration helpers, not
  first-class rule-side effect execution
- the published `v1.1.1` line does not claim full rule-side effect execution
  semantics as part of its stable commitment

That stable reading remains the source of truth until this track explicitly
lands a widened post-stable contract.

## Included In This Track

- explicit owner-layer for rule-side effect declarations and effect plans
- deterministic execution contract for an admitted first-wave effect subset
- matching state/audit/runtime integration for the same admitted effect subset
- tests/docs/spec/runtime-validation sync for the widened rule execution
  contract

## Explicit Non-Goals

- widening host ABI or capability policy by implication
- reopening the published `v1.1.1` line as if full rule execution already
  shipped there
- distributed scheduling, async workers, or background job orchestration
- persistence/replay widening beyond the already completed persisted archive
  track
- implicit retry, rollback, or compensation semantics
- full generic effect system or user-defined effect families

## Intended Slice Order

1. docs/governance checkpoint
2. explicit rule-effect ownership in `prom-rules`
3. deterministic execution for one narrow in-process effect family
4. deterministic execution for one narrow audit-facing effect family
5. docs/spec/runtime-validation freeze for the widened rule execution contract

## Acceptance Reading

This track is done only when:

- admitted rule-side effects have explicit owner-layer representation
- rule evaluation, effect execution, and audit/state integration agree on the
  same deterministic semantics
- release-facing docs distinguish the published `v1.1.1` baseline from the new
  post-stable widened contract
- no part of the work quietly widens host behavior, persistence/replay, or
  recovery semantics
