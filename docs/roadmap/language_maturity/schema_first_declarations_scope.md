# Schema-First Declarations Scope

Status: first-wave implemented baseline history
Related issue: `#121`

Implementation note:

- this scope is already implemented on current `main`
- the planning language below is retained as the historical acceptance contract
  for the landed first wave

## Goal

Define a narrow, executable first-wave contract for schema-first declarations
without accidentally opening validation generation, config loading, API codegen,
migrations, or PROMETHEUS boundary work inside the first `v0.3` slice.

## Decision

`#121` should start as a declaration-surface wave, not as a generation or
runtime wave.

The first honest target is a canonical schema declaration family that can
describe:

- product-shaped models
- tagged union models
- schema roles for later config/API/wire use

while remaining separate from:

- executable record / ADT runtime carriers
- generated validation logic
- host integration
- serialization / migration machinery

## Included In `#121`

- top-level nominal schema declarations as language-owned source items
- canonical schema forms for:
  - record-shaped schemas
  - tagged-union schemas
  - role markers for `config`, `api`, and `wire`
- reuse of the current declared type grammar inside schema field / payload
  positions
- clean profile or edition gating so schema declarations can exist without
  widening unrelated execution surfaces
- deterministic frontend ownership of schema tables and diagnostics

## Explicit Non-Goals

- deriving validation or runtime checks from schemas
- config-file loading or parsing
- generated client/server artifacts
- schema versioning or migration metadata
- patch types or wire-format unions beyond the declaration layer
- widening `prom-*`, host capability scope, or executable VM value families
- silently treating schemas as executable records or ADTs

## Honest First-Wave Rules

- schema declarations are compile-time contract items, not executable value
  families
- schema declarations may reference already-supported declared source types,
  including records, tuples, nominal enums, `Option(T)`, `Result(T, E)`, and
  measured numeric forms
- schema declarations must stay nominal and deterministic; duplicate names or
  ambiguous shape forms are compile-time errors
- config/API/wire meaning is introduced only as explicit schema-role metadata,
  not through ad hoc naming conventions
- `#121` is not done when only one shape family exists; records, tagged unions,
  and schema-role markers are all part of the acceptance boundary

## Expected Slice Order

1. schema record declarations and canonical schema table ownership
2. schema tagged-union declarations
3. schema role markers for `config`, `api`, and `wire`
4. docs/spec freeze for declaration-only schema surface

## Done Boundary

`#121` can close when:

1. canonical schema declaration syntax exists for record and tagged-union
   shapes,
2. schema-role markers for `config`, `api`, and `wire` are explicit and
   deterministic,
3. schema declarations stay compile-time-only and separate from executable
   runtime carriers,
4. docs and diagnostics describe the declaration surface honestly,
5. the implementation remains inside `sm-front` / `sm-profile` / `sm-sema` /
   `sm-ir` and does not widen PROMETHEUS or host boundaries.
