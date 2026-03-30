# Generated API Contract Surface Scope

Status: proposed

## Goal

Turn canonical `api schema` and `wire schema` declarations into one deterministic,
reproducible generated API contract artifact owned by `smc-cli`, without
introducing hand-maintained duplicates beside the schema table.

## Why

`V03-01` established canonical schema declarations and role markers.
`V03-02` established deterministic validation plans derived from those schemas.
`V03-04` should now generate one inspectable API-contract artifact from that
same canonical source of truth, so users can version and review boundary
contracts without maintaining a second parallel API description layer.

## First-Wave Scope

- derive generated API contract artifacts only from canonical `api schema` and
  `wire schema` declarations
- keep artifact generation deterministic and declaration-order preserving
- make generated outputs inspectable, versioned, and stable enough for checked-in
  review
- keep ownership in `smc-cli`
- avoid any new hand-authored API description format

## Intended First-Wave Artifact Shape

- one canonical text artifact format owned by `smc-cli`
- record-shaped schemas emit object fields in declaration order
- tagged-union schemas emit variant branches in declaration order
- role metadata stays explicit in the generated artifact
- output includes explicit generator/version metadata for reproducibility

## Intended Slice Order

1. generated API contract scope checkpoint
2. canonical API contract artifact model and formatter ownership
3. deterministic generation for record-shaped schemas
4. deterministic generation for tagged-union schemas
5. diagnostics/docs freeze for generated API contract artifacts

## Slice-2 Contract Reading

The first code slice for `V03-04` owns only the canonical generated artifact
shape and formatter in `smc-cli`.

- it introduces one explicit artifact family and one formatter surface
- it preserves declaration order supplied by later derivation slices
- it does not yet derive artifacts from canonical schemas

## Non-Goals

- emitting client SDKs or server stubs
- runtime transport integration
- schema migrations
- config loading
- widening `prom-*`, host capability, or VM/runtime boundaries
- introducing a second editable API truth layer

## Acceptance Reading

This issue is done only when:

- canonical schemas generate one stable API contract artifact family
- generated outputs are deterministic and versioned
- there is no hand-maintained duplicate API description layer
- user-facing artifact expectations are documented clearly enough for first-wave
  review and version control
