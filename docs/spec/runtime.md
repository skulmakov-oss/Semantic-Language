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

## Ownership Rule

`prom-runtime` owns:

- execution session model
- verified program loading path
- wiring of host, capability, and gate contexts into an execution session

`prom-runtime` does not own:

- ABI descriptor semantics
- capability policy semantics
- gate registry semantics
- semantic state, agenda, or rule scheduling

## Session Rule

Current session invariant:

- every runtime session must execute only through verified SemCode entrypoints
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

## Controlled Integration Surface

Current narrow orchestration helpers:

- state to agenda derivation through `RuleEngine::evaluate`
- deterministic next-activation selection through `ActivationSelection`
- state update application through `SemanticStateStore::apply` followed by agenda refresh
- canonical audit emission helpers for:
  - session start and finish
  - rule activation
  - state transition metadata

These helpers are orchestration glue only. They must not redefine:

- state validation rules
- agenda ordering rules
- audit event schema ownership
