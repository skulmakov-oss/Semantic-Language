# Validation Derived From Schemas Scope

Status: first-wave implemented baseline history
Related issue: `#122`

Implementation note:

- this scope is already implemented on current `main`
- canonical schema validation-plan ownership and generated validation checks are
  already part of the current tooling surface
- the planning language below is retained as the historical acceptance contract
  for the landed first wave

## Goal

Define a narrow, inspectable first-wave contract for validation derived from
canonical schema and declared type information without accidentally opening
config loading, API code generation, migrations, wire serialization, or
PROMETHEUS boundary work inside the next `v0.3` slice.

## Decision

`#122` should start as a deterministic validation-derivation wave, not as a
runtime ingestion or artifact-generation wave.

The first honest target is a canonical validation plan owned by the language
and sema layers that can be derived from:

- record-shaped schemas
- tagged-union schemas
- already-supported declared source types referenced from schema fields

while remaining separate from:

- config-file readers and environment loaders
- generated server/client code
- migrations and schema evolution policies
- host integration or PROMETHEUS runtime hooks

## Included In `#122`

- canonical validation-plan ownership for schema declarations inside the
  compile-time analysis path
- deterministic traversal of schema shapes and referenced declared types
- first-wave generated validation checks for:
  - required fields
  - duplicate or impossible tagged-union branch states
  - declared type compatibility at schema leaf positions
- inspectable validation output shape suitable for docs and CLI explanation
- canonical diagnostics for generated validation failures

## Explicit Non-Goals

- loading config files, HTTP payloads, or wire messages
- emitting generated Rust or Semantic source artifacts
- schema migrations or version negotiation
- patch application semantics
- host capability or `prom-*` boundary widening
- introducing new executable VM value families for schema validation
- treating schema-derived validation as implicit runtime behavior

## Honest First-Wave Rules

- validation derivation is compile-time-owned and deterministic
- derived validation rules must be inspectable and attributable to canonical
  schema declarations
- the first wave may reuse already-supported declared source types, but it does
  not introduce general user-defined generics or arbitrary predicate DSLs
- generated validation failures must remain ordinary language diagnostics, not
  hidden runtime traps
- `#122` is not done when only schema storage exists; deterministic derivation,
  inspectable output, and diagnostic coverage are all part of the acceptance
  boundary

## Expected Slice Order

1. validation-plan ownership for canonical schemas and declared types
2. deterministic derivation for record-shaped schemas
3. deterministic derivation for tagged-union schemas
4. inspectable output and diagnostics freeze for generated validation failures

## Done Boundary

`#122` can close when:

1. canonical schema declarations derive deterministic validation plans,
2. derived validation stays inspectable and attributable to source schemas,
3. generated validation failures are documented and emitted through ordinary
   diagnostics paths,
4. the implementation remains inside `sm-sema` / `sm-front` / `sm-ir` /
   `smc-cli`,
5. no config loading, artifact generation, migration logic, or PROMETHEUS /
   host boundary widening is introduced.
