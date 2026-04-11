# Runtime Ownership Specification

Status: draft v0
Source ownership owner: `sm-front`
IR ownership owner: `sm-ir`
SemCode transport owner: `sm-ir`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`
Shared runtime vocabulary owner: `sm-runtime-core`

## Purpose

This document freezes the current first-wave runtime ownership contract.

Current supported slice:

- tuple-only `AccessPath`
- frame-local borrow lifetime
- structural `OWN0` admission before execution
- runtime write rejection for overlapping borrowed tuple paths

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

Current ordering rule:

- path components are ordered from root to leaf
- the same path must serialize, admit, and execute in the same deterministic
  order

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

## Frontend And Lowering Contract

Current source/frontend contract:

- tuple borrow capture must not be erased before lowering
- lowering must preserve enough ownership metadata to recover:
  - borrow event kind
  - write event kind
  - canonical tuple-only `AccessPath`

Current lowering contract:

- runtime ownership transport is path-based, not AST-pattern-based
- the lowered contract uses canonical `AccessPath` rooted in `SymbolId`

## SemCode Transport Contract

Current binary contract:

- ownership metadata is transported only through `SEMCOD11`
- the ownership section tag is `OWN0`
- each event carries:
  - event kind (`Borrow` or `Write`)
  - root `SymbolId`
  - ordered tuple-only path components

Current transport scope:

- tuple-only path components
- deterministic event order

## Verifier Admission Contract

Current verifier responsibility:

- validate `OWN0` section structure
- validate admitted ownership event kinds
- validate tuple-only path payload shape
- validate header/capability consistency for ownership transport

Current verifier non-goal:

- do not evaluate borrow overlap policy
- do not execute runtime ownership semantics

## VM Enforcement Contract

Current VM responsibility:

- keep a frame-local set of active borrowed tuple paths
- consume admitted ownership metadata only
- reject overlapping writes at runtime for the supported tuple slice

Current VM non-goals:

- no partial borrow release
- no inter-frame borrow persistence
- no advanced alias inference

## Explicitly Unsupported

The current runtime ownership contract does not claim support for:

- record field paths
- ADT payload paths
- schema paths
- partial borrow release before frame exit
- advanced aliasing or region reasoning
- inter-frame borrow persistence
- non-tuple ownership transport

## Honesty Rule

If a behavior is not implemented across:

- lowering
- SemCode transport
- verifier admission
- VM enforcement

then it must remain unsupported here rather than being implied by analogy.
