# PROMETHEUS Runtime Orchestration Specification

Status: draft v0
Owner crate: `prom-runtime`

This document defines the current orchestration contract for running verified Semantic programs inside PROMETHEUS-facing execution sessions.

## Current Runtime Surface

Current canonical orchestration types:

- `RuntimeSessionDescriptor`
- `ExecutionSession`
- `GateExecutionSession`
- `ActivationSelection`
- `RuntimeStateAdvance`
- `RuleStateWriteAdvance`
- `RuleAuditNoteAdvance`
- `RuleEffectExecutionError`

## Ownership Rule

`prom-runtime` owns:

- execution session model
- verified program loading path
- wiring of host, capability, and gate contexts into an execution session

`prom-runtime` does not own:

- ABI descriptor semantics
- capability policy semantics
- verifier admission
- VM execution mechanics
- runtime trap taxonomy
- quota semantics
- gate registry semantics
- semantic state, agenda, or rule scheduling

Current `v1` rule:

- richer semantic runtime semantics remain non-blocking for `v1` while the PROMETHEUS boundary stays on the narrow ABI/capability/gate surface

## Session Rule

Current session invariant:

- every runtime session must execute only through verified SemCode entrypoints
- raw or testing-only VM helpers are not part of the public orchestration
  boundary
- session context is explicit through `ExecutionConfig` / `ExecutionContext`
- session descriptor must expose:
  - execution context
  - capability manifest metadata
  - whether a gate registry is bound

## Boundary Rule

Current orchestrator wiring:

- `ExecutionSession` wires a generic `prom-abi` host and `prom-cap` checker
- `GateExecutionSession` wires `prom-gates` through `GateHostAdapter`
- session orchestration may compose owner crates, but it must not redefine their contracts
- orchestration must surface VM verifier rejection and runtime traps as owned by
  the execution layer, not reinterpret them as new orchestration semantics

## Controlled Integration Surface

Current narrow orchestration helpers:

- state to agenda derivation through `RuleEngine::evaluate`
- deterministic next-activation selection through `ActivationSelection`
- state update application through `SemanticStateStore::apply` followed by agenda refresh
- first-wave admitted rule-side effect execution for ordered `RuleEffect::StateWrite`
  plans only
- second admitted audit-facing rule-side effect execution for ordered
  `RuleEffect::AuditNote` plans only
- canonical audit emission helpers for:
  - session start and finish
  - rule activation
  - state transition metadata
  - rule audit notes
- persisted archive creation remains delegated to owner crates:
  - `prom-state` for `StateSnapshotArchive`
  - `prom-audit` for `AuditReplayArchive`

Current admitted execution-value reading:

- current `main` now executes admitted `text` literal/equality programs through
  the verified SemCode path
- this carrier remains internal to verified Semantic execution
- the PROMETHEUS host ABI still does not admit text values
- current `main` now also executes one ordered sequence family through the
  verified SemCode path, including literal materialization, same-family
  equality, and `expr[index]`
- this sequence carrier remains internal to verified Semantic execution
- the PROMETHEUS host ABI still does not admit sequence values
- current `main` now also executes one first-wave closure family through the
  verified SemCode path, including immutable capture materialization and direct
  invocation with exactly one positional argument
- this closure carrier remains internal to verified Semantic execution
- the PROMETHEUS host ABI still does not admit closure values

These helpers are orchestration glue only. They must not redefine:

- state validation rules
- agenda ordering rules
- audit event schema ownership

## Current Rule Effect Execution Boundary

Current first-wave admitted execution family:

- `RuleEffect::StateWrite`
  - executes in declared order
  - materializes into canonical `StateUpdate`
  - refreshes agenda after every applied transition
  - emits only canonical `AuditEventKind::StateTransition` entries

Current second admitted execution family:

- `RuleEffect::AuditNote`
  - executes in declared order
  - emits canonical `AuditEventKind::Note` entries only
  - does not mutate state
  - does not refresh agenda

Current admitted slices explicitly still do not provide:

- implicit retries, rollback, or compensation semantics
- mixed-family effect execution through a generic rule-effect engine
