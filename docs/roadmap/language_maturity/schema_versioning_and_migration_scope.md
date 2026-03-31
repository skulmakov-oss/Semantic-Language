# Schema Versioning And Migration Scope

Status: proposed

## Goal

Add one canonical, diffable versioning and migration-metadata layer on top of
the existing schema surface, without introducing runtime migration execution or
a second editable schema truth source.

## Why

`V03-01` established canonical schema declarations and role markers.
`V03-04` established deterministic generated API contract artifacts from
canonical `api schema` and `wire schema` declarations. `V03-05` should now make
schema evolution explicit enough for review: version identity, compatibility
reading, and migration metadata should be visible, deterministic, and testable
without widening runtime or host boundaries.

## First-Wave Scope

- explicit schema version metadata owned by the canonical schema table
- deterministic compatibility reading between adjacent schema versions
- migration metadata as inspectable compile-time contract data
- stable additive-versus-breaking documentation for first-wave review
- ownership kept inside frontend/schema tooling layers rather than runtime

## Intended First-Wave Shape

- one canonical version marker per schema declaration family
- explicit compatibility categories for:
  - additive field or variant growth
  - incompatible removals or type changes
- deterministic migration metadata representation owned by tooling
- diff/review output derived only from canonical schema declarations

## Intended Slice Order

1. schema versioning scope checkpoint
2. canonical schema-version metadata ownership
3. deterministic compatibility classification for record-shaped schemas
4. deterministic compatibility classification for tagged-union schemas
5. migration metadata formatting and docs freeze

## Slice-2 Contract Reading

The first code slice owns only canonical schema-version metadata.

- `schema` declarations may now attach optional `version(<u32>)` metadata
- version metadata is retained in the canonical schema table
- this slice does not yet derive compatibility classes or migration plans

## Slice-3 Contract Reading

The second code slice derives only deterministic compatibility classification
for record-shaped schemas across two explicit schema versions.

- both compared schemas must carry explicit version metadata
- this slice currently classifies only `Equivalent`, `Additive`, or `Breaking`
- field additions are additive; field removals or field-type changes are
  breaking
- tagged-union compatibility and migration metadata remain deferred

## Slice-4 Contract Reading

The third code slice extends deterministic compatibility classification to
tagged-union schemas across two explicit schema versions.

- both compared schemas must carry explicit version metadata
- this slice still classifies only `Equivalent`, `Additive`, or `Breaking`
- variant additions and payload-field additions are additive
- variant removals, payload-field removals, and payload type changes are
  breaking
- migration metadata remains deferred

## Non-Goals

- runtime migration execution
- config loading
- client/server transport integration
- schema patch application engines
- widening `prom-*`, host capability, or VM/runtime boundaries
- introducing a second hand-maintained migration truth layer

## Acceptance Reading

This issue is done only when:

- schema versions are explicit and inspectable
- compatibility reading is deterministic and diffable
- migration metadata is documented as compile-time/tooling contract data
- additive versus breaking evolution is documented clearly enough for stable
  first-wave review
