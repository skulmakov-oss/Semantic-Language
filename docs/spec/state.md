# PROMETHEUS Semantic State Specification

Status: draft v0
Owner crate: `prom-state`

This document defines the current canonical semantic state model for PROMETHEUS-facing reasoning state.

## Current State Surface

Current canonical state types:

- `FactValue`
- `FactResolution`
- `ContextWindow`
- `StateEpoch`
- `StateRecord`
- `StateUpdate`
- `StateSnapshot`
- `StateSnapshotArchive`
- `StateTransitionMetadata`
- `SemanticStateStore`

## Ownership Rule

`prom-state` owns:

- semantic fact records
- uncertainty and conflict representation
- context window attachment
- epoch/version model for state evolution
- snapshot and restore surface
- persisted snapshot archive envelope shape
- state validation invariants

`prom-state` does not own:

- VM execution mechanics
- ABI or capability policy
- rule activation and agenda logic
- orchestration lifecycle

## Validation Rule

Current state invariants:

- state keys must not be empty
- context windows must not be empty
- transition reasons must not be empty
- `Certain` holds exactly one value
- `Uncertain` and `Conflicted` require at least two unique alternatives

## Transition Rule

Current store behavior:

- every accepted update advances the epoch exactly once
- every accepted update produces explicit transition metadata
- snapshots capture the full visible state at a specific epoch
- restore replaces visible state with the selected snapshot state

## Persistence Rule

Current persisted state rule:

- `StateSnapshotArchive` wraps one canonical `StateSnapshot`
- archive metadata is explicit through `format_version`
- archive materialization/loading uses one canonical deterministic text envelope
- persisted archive ownership does not widen store validation or runtime
  recovery semantics by implication

## Boundary Rule

Current architectural rule:

- `prom-runtime` may orchestrate execution around state, but it must not become the owner of state schema or validation rules
- `prom-rules` may later consume this state model, but it must not redefine state storage invariants
