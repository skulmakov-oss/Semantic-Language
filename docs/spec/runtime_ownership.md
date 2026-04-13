# Runtime Ownership Specification

Status: draft v0
Source ownership owner: `sm-front`
IR ownership owner: `sm-ir`
SemCode transport owner: `sm-ir`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`
Shared runtime vocabulary owner: `sm-runtime-core`

## Purpose

This document freezes the current runtime ownership contract and the next
approved extension boundary for direct record field paths.

Current supported slice:

- tuple-only `AccessPath`
- frame-local borrow lifetime
- structural `OWN0` admission before execution
- runtime write rejection for overlapping borrowed tuple paths

Approved next slice for this track:

- direct record field `AccessPath`
- borrow and write ownership over direct record field paths
- the same overlap rule family used by the tuple-only slice

This document does not claim a general runtime borrow checker.

## Layer Separation

The current ownership pipeline is intentionally split:

- source/frontend semantics decide where borrow capture exists in source
- IR/lowering preserves only the canonical execution-path contract
- SemCode transports that lowered ownership metadata in `OWN0`
- verifier admits or rejects the `OWN0` payload structurally
- VM enforces the runtime write-path guard over admitted tuple paths

Important rule:

- runtime ownership must not depend on frontend AST or parser-only pattern
  structures

## Canonical Runtime Path

Current runtime path form:

- `AccessPath { root: SymbolId, components: Vec<PathComponent> }`

Current supported component kinds:

- `TupleIndex(u16)`

Approved next component kind for this track:

- `Field(SymbolId)` for direct named record field projection only

Current ordering rule:

- path components are ordered from root to leaf
- the same path must serialize, admit, and execute in the same deterministic
  order

Important boundary:

- this document does not approve indirect field selection or broader path
  normalization

## Supported Behavior

Current supported runtime ownership behavior is limited to tuple paths.

Borrow lifetime v0:

- a borrowed tuple path becomes active for the current frame
- the active borrowed-path set is cleared when that frame exits

Current runtime write rule:

- a write must be rejected if its target path overlaps an active borrowed path

Current overlap cases that must reject:

- exact path equality
- borrowed parent, written child
- borrowed child, written parent

Current allowed case:

- sibling tuple paths

## Approved Record Field Extension Scope

The next runtime ownership slice approved by this document extends the current
tuple-only contract to direct record field access paths.

Approved target behavior for that track:

- borrow direct record fields
- write direct record fields
- reject exact overlap
- reject borrowed parent, written child
- reject borrowed child, written parent
- allow sibling record fields

Approved identity rule for that track:

- record field path identity is represented as `Field(SymbolId)` inside
  `AccessPath`
- the first slice is limited to direct named field projection

## Frontend And Lowering Contract

Current source/frontend contract:

- tuple borrow capture must not be erased before lowering
- lowering must preserve enough ownership metadata to recover:
  - borrow event kind
  - write event kind
  - canonical tuple-only `AccessPath`

Approved next frontend/lowering target:

- preserve borrow and write intent for direct record field access paths
- lower direct record field paths into canonical `AccessPath` using
  `Field(SymbolId)`

Current lowering contract:

- runtime ownership transport is path-based, not AST-pattern-based
- the lowered contract uses canonical `AccessPath` rooted in `SymbolId`

## SemCode Transport Contract

Current binary contract:

- tuple-only ownership metadata is transported through `SEMCOD11`
- direct record-field `Borrow`/`Write` transport is emitted through `SEMCOD12`
- the ownership section tag is `OWN0`
- each event carries:
  - event kind (`Borrow` or `Write`)
  - root `SymbolId`
  - ordered path components

Current transport scope:

- tuple-only path components admitted end-to-end
- direct record-field `Borrow`/`Write` transport, encoded as `Field(SymbolId)`
- deterministic event order

Approved next transport scope:

- deterministic transport of direct record field path components through the
  same ownership event contract

## Verifier Admission Contract

Current verifier responsibility:

- validate `OWN0` section structure
- validate admitted ownership event kinds
- validate tuple and direct record-field path payload shape
- validate header/capability consistency for ownership transport

Approved next verifier scope:

- admit or reject direct record field ownership payload structurally
- do not imply ADT payload or schema ownership support

Current verifier non-goal:

- do not evaluate borrow overlap policy
- do not execute runtime ownership semantics

## VM Enforcement Contract

Current VM responsibility:

- keep a frame-local set of active borrowed tuple paths
- consume admitted ownership metadata only
- reject overlapping writes at runtime for the supported tuple slice

Approved next VM target:

- track borrowed direct record field paths in the same frame-local ownership
  model
- reject overlapping writes for admitted direct record field paths

Current VM non-goals:

- no partial borrow release
- no inter-frame borrow persistence
- no advanced alias inference

## Explicitly Unsupported

The current implemented runtime ownership contract does not claim support for:

- record field paths until the direct-field track is landed
- ADT payload paths
- schema paths
- partial borrow release before frame exit
- advanced aliasing or region reasoning
- inter-frame borrow persistence
- non-tuple ownership transport

The approved record-field track still remains explicitly out of scope for:

- ADT payload paths
- schema paths
- partial release before frame exit
- advanced aliasing or region reasoning
- inter-frame borrow persistence
- indirect field selection or broader smart path normalization

## Honesty Rule

If a behavior is not implemented across:

- lowering
- SemCode transport
- verifier admission
- VM enforcement

then it must remain unsupported here rather than being implied by analogy.
