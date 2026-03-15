# Logos Surface Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current declarative Logos-oriented source surface used
for system, entity, and law descriptions inside Semantic.

It complements the Rust-like executable surface described in `syntax.md`.

## Current Top-Level Forms

The current Logos surface recognizes these top-level declarations:

- `System`
- `Entity`
- `Law`

Current legacy compatibility directives may still exist in guarded paths, but
they are not the primary long-term public contract for the Logos surface.

## System

Current `System` form:

```sm
System Name(param: type, ...)
```

Current rule:

- `System` declares one top-level system descriptor
- parameters are explicit and typed

## Entity

Current `Entity` form:

```sm
Entity Sensor:
    state val: quad
    prop threshold: f64
```

Current entity-field kinds include:

- `state`
- `prop`

Current rule:

- entity bodies are indentation-delimited
- each field has a kind, a name, and a type

## Law

Current `Law` form:

```sm
Law "CheckSignal" [priority 10]:
    When Sensor.val == T -> Log.emit("Signal OK")
```

Current law properties:

- law names are string-literal based
- `priority` is optional and numeric
- law bodies are indentation-delimited
- law bodies contain one or more `When` clauses

## When Clauses

Current `When` form:

```sm
When condition -> effect
```

Current rule:

- empty `When` conditions are rejected
- empty `When` effects are rejected
- the current frontend stores condition and effect as structured text fragments
  at this surface, not as the Rust-like executable AST

## Ordering Rule

Current rule:

- parsed laws are ordered by descending priority in the current Logos program

This behavior is part of the current public source contract and should not be
changed silently.

## Policy Rule

The Logos surface is policy-gated:

- it may be disabled by the active parser profile
- profile rejection is a source-level policy violation, not a runtime error

## Current Limits

The current Logos contract does not yet claim stable support for:

- a fully separate package or module ecosystem for Logos-only projects
- rich user-defined statement semantics inside `When` beyond the current
  text-fragment contract
- broad legacy directives as first-class long-term source features

## Contract Rule

Any public change to `System`, `Entity`, `Law`, `When`, field kinds, priority
ordering, or profile-gating behavior should update this document in the same
change series.
