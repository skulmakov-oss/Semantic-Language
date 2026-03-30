# Config Schema Contract Scope

Status: proposed

## Goal

Turn `config schema` declarations plus derived validation plans into one
canonical config-file contract owned by `smc-cli`, without introducing a second
truth layer beside the schema table.

## Why

`V03-01` established canonical schema declarations and role markers.
`V03-02` established deterministic validation plans derived from those schemas.
`V03-03` should now bind that compile-time truth to one canonical config-file
surface so users can parse and validate config documents without inventing
parallel ad hoc config rules.

## First-Wave Scope

- treat `config schema` as the only schema-role entry point for this wave
- choose one canonical config document surface owned by `smc-cli`
- parse canonical config documents into an inspectable intermediate config model
- validate parsed config documents against the canonical validation-plan table
- keep validation deterministic and declaration-order preserving
- make diagnostics user-facing and stable at the config-contract layer

Current chosen canonical config document surface:

- root object only
- identifier keys
- nested object values
- scalar values limited to string, bool, quad, and decimal/integer numbers
- no arrays, comments, or alternate wire/config syntaxes in the first slice

Current second-slice validation boundary:

- validate only record-shaped `config schema` roots
- allow nested record fields via canonical record declarations
- allow measured numeric fields through unit-erased numeric compatibility checks
- keep tagged-union config validation for a later slice

## Intended Slice Order

1. config contract scope checkpoint
2. canonical config document model and parser ownership in `smc-cli`
3. record-shaped config schema parse/validation path
4. tagged-union config schema parse/validation path if still required by the
   issue acceptance boundary
5. diagnostics/docs freeze for the canonical config contract

## Non-Goals

- supporting multiple config formats in the first wave
- introducing runtime config loading semantics
- generating code, loaders, or transport artifacts
- widening `prom-*`, host capability, or VM/runtime boundaries
- creating an alternate config truth layer separate from canonical schemas and
  validation plans

## Acceptance Reading

This issue is done only when:

- one canonical config document surface exists
- config parsing and validation reuse canonical schema and validation-plan
  ownership
- user-facing diagnostics for config validation are documented and stable enough
  for the first-wave contract
- the implementation does not duplicate schema truth in a second hand-written
  config model
