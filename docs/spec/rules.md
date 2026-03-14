# PROMETHEUS Rule Runtime Specification

Status: draft v0
Owner crate: `prom-rules`

This document defines the current canonical rule model and deterministic agenda contract for Semantic runtime reasoning.

## Current Rule Surface

Current canonical rule types:

- `RuleId`
- `Salience`
- `RuleCondition`
- `RuleDefinition`
- `AgendaEntry`
- `Agenda`
- `RuleEngine`

## Ownership Rule

`prom-rules` owns:

- rule identity and validation
- rule condition model
- salience and deterministic agenda ordering
- rule activation evaluation against semantic state

`prom-rules` does not own:

- state storage invariants
- VM execution mechanics
- host ABI or capability policy
- orchestration session lifecycle

## Activation Rule

Current v0 activation semantics:

- rules are evaluated against `prom-state`
- a condition matches only when the referenced fact exists and is `Certain(expected_value)`
- `Uncertain` and `Conflicted` state does not activate certain-match rules

## Agenda Rule

Current deterministic ordering:

- higher salience activates first
- equal salience preserves rule registration order
- remaining ties are ordered by rule identity

## Boundary Rule

Current architectural rule:

- `prom-rules` may consume `prom-state`, but it must not redefine state schema or validation
- `prom-runtime` may orchestrate agenda execution later, but it must not become the owner of rule model or scheduling semantics
