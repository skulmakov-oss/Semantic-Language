# User Data Model Expansion

Status: proposed v0

## Goal

Expand Semantic from a scalar- and quad-oriented reasoning core into a
language with an intentional user-facing data model.

This workstream is about broadening what users can model directly in source
programs, not about replacing the current deterministic compiler and VM
pipeline.

## Current Baseline

Today the executable Rust-like surface is intentionally narrow.

The current user-visible value families are:

- `quad`
- `bool`
- `i32`
- `u32`
- `f64`
- `fx`
- `unit`
- `qvec(N)` as reserved/partial rather than broadly stabilized

Current honest constraints:

- there are no user-defined aggregate executable types
- there are no tuples as a first-class stabilized language surface
- there are no strings or collection types as part of the executable public
  contract
- Logos `Entity` declarations are domain-level descriptors, not general-purpose
  executable records

## Why This Matters

Semantic is already a real language and toolchain, but it is still narrow
compared with languages such as Rust or Python because users cannot yet model
ordinary structured domain data directly.

Without an intentional user data model, users are forced to:

- split related values across many scalar bindings
- encode structure through naming conventions instead of types
- keep domain state in ad hoc parallel locals
- rely on Logos declarations for concerns that belong to ordinary executable
  data

## Design Principles

The user data model should preserve the existing platform character.

Required principles:

- deterministic lowering
- explicit construction
- predictable layout
- no silent dynamic runtime object model
- compatibility with `quad`-oriented reasoning
- compatibility with verifier-before-execution architecture

The data model should broaden expressiveness without importing Python-style
dynamic semantics or a hidden heap-first execution model.

## Staged Expansion Plan

### Stage 1: Records

The first intentional aggregate family should be nominal records.

Reason:

- records solve the largest real user pain with the smallest semantic blast
  radius
- they compose naturally with existing function, equality, and field-oriented
  reasoning
- they can lower deterministically without requiring a general object runtime

The detailed design for this stage is defined in:

- `docs/roadmap/language_maturity/record_data_model.md`
- `docs/roadmap/language_maturity/record_scenarios.md`

### Stage 2: Tuples

After records, the next likely family is tuples.

Intent:

- compact positional product values for helper returns and local grouping
- no field names, only positional structure
- narrower semantics than records

### Stage 3: Tagged Unions / Enums

After product types, the next likely expansion is a tagged sum family.

Intent:

- explicit variant modeling
- richer `match` surface beyond `quad`
- deterministic tagged lowering

This stage should be designed only after Stage 1 record semantics are stable.

### Stage 4: Collections And Strings

Strings and collections should come after aggregate product/sum types, not
before them.

Intent:

- sequence and map-like containers
- textual data as a first-class source family
- library-backed collection semantics rather than accidental frontend sugar

This stage should not be treated as a near-term blocker for the first language
maturity wave.

## Non-Goals

This workstream is not intended to:

- replace the deterministic execution model
- introduce Python-style dynamic typing
- make broad post-`v1` runtime work an immediate blocker
- promise a full heap/object runtime before the first aggregate family lands

## Acceptance Criteria

This workstream should be considered materially started only when:

- the repository has a documented staged user-data-model roadmap
- at least one intentional aggregate family is specified end-to-end
- examples show meaningful structured user data beyond scalar-only flows
- typing and lowering expectations are documented rather than left implicit

## Immediate Next Slice

The immediate next slice for this PR is Stage 1 records, because it is the
smallest coherent step that turns "broader data model" into a concrete language
direction.
