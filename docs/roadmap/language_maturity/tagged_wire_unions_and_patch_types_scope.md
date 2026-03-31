# Tagged Wire Unions And Patch Types Scope

Status: proposed

## Goal

Turn canonical `wire schema` declarations into one deterministic, inspectable
wire-contract family for tagged unions and patch/partial update shapes, without
introducing a second editable serialization model.

## Why

`V03-01` established canonical schema declarations and explicit role markers.
`V03-04` established generated API-contract artifacts as reviewable boundary
outputs. `V03-06` should now define the narrow wire-shape contract that sits on
top of canonical `wire schema` declarations, so update payloads and tagged wire
branches stay explicit, deterministic, and reviewable.

## First-Wave Scope

- derive tagged wire-union metadata only from canonical tagged-union
  `wire schema` declarations
- derive patch/partial shapes only from canonical record-shaped `wire schema`
  declarations
- keep all emitted wire-contract families deterministic and declaration-order
  preserving
- keep ownership in `smc-cli`
- document serialization/update semantics explicitly enough for review and tests

## Intended First-Wave Contract Shape

- tagged wire unions preserve declaration-order variant branches and explicit
  payload field order
- record patch types expose optional update entries per declared field in
  declaration order
- patch types model partial update intent only; they do not imply a runtime
  patch application engine
- wire-contract artifacts remain inspectable, versionable, and derived from the
  canonical schema table

## Intended Slice Order

1. tagged wire unions and patch types scope checkpoint
2. canonical wire-contract artifact ownership for tagged unions and patch types
3. deterministic tagged wire-union derivation
4. deterministic record patch-type derivation
5. diagnostics/docs freeze for wire-contract review semantics

## Slice-2 Contract Reading

The first code slice for `V03-06` owns only the canonical artifact/model layer
for tagged wire unions and patch types.

- it introduces one explicit owner surface in `smc-cli`
- it defines stable formatter/review expectations
- it freezes one canonical artifact family for wire unions and wire patch types
- it does not yet derive artifacts from canonical schemas

## Slice-3 Contract Reading

The second code slice derives tagged wire unions only from canonical tagged-
union `wire schema` declarations.

- it preserves variant and payload-field declaration order
- it still leaves record-shaped patch derivation for the next slice
- `api schema`, `config schema`, and unmarked schemas remain outside this
  derivation path

## Slice-4 Contract Reading

The third code slice derives record patch types only from canonical
record-shaped `wire schema` declarations.

- patch fields stay in declaration order
- every emitted patch entry remains explicit review metadata, not runtime patch
  behavior
- tagged wire-union derivation remains unchanged and stable

## Non-Goals

- migration execution
- runtime patch application
- client/server code generation
- config loading
- schema validation derivation
- widening `prom-*`, host capability, or VM/runtime boundaries
- introducing a second editable wire-description layer

## Acceptance Reading

This issue is done only when:

- tagged wire unions derive deterministically from canonical declarations
- patch/partial shapes derive deterministically from canonical record-shaped
  `wire schema` declarations
- update semantics remain explicit in docs and tests
- wire review artifacts stay stable and inspectable enough for checked-in
  review
- there is no implicit runtime patch engine or widened host/runtime boundary
