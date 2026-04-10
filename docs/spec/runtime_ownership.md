# Runtime Ownership Path Specification

Status: draft v0 contract freeze
Primary execution owners: `sm-ir`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`
Related frontend owners: `sm-front`, `sm-sema`

## Purpose

This document freezes the minimal execution-layer ownership contract required
for deterministic runtime rejection of writes that overlap active borrows.

It defines a narrow runtime-observable ownership slice. It does not turn the
VM into a general borrow checker.

This document exists because source-level borrow semantics and runtime
execution must remain separate:

- frontend ownership uses source-oriented binding and pattern semantics
- lowering must translate those semantics into an execution contract
- SemCode must transport that contract explicitly
- VM must execute only the lowered contract, not frontend AST structures

Current `main` does not yet fully transport this contract end-to-end. This
document freezes the target v0 shape for the implementation series that will
land that support.

## Stable Boundary Rule

Runtime ownership metadata must follow these boundary rules:

- compile-time ownership and runtime ownership are related but not identical
- VM must not depend on parser, AST, or frontend pattern objects
- execution must use a lowered canonical path descriptor
- verifier checks payload structure and admissibility, not borrow semantics

## Runtime Ownership Goal

The v0 runtime goal is:

- if a write target path overlaps any active borrowed path, execution rejects
  the write deterministically

Current overlap cases:

- exact path overlap rejects
- parent of borrowed child path rejects
- child of borrowed parent path rejects
- sibling paths remain allowed

## AccessPath v0

The canonical execution-layer path form is `AccessPath`.

Current v0 shape:

- one root variable identity: `SymbolId`
- an ordered component list relative to that root

Current v0 component family:

- `TupleIndex(u16)`

`AccessPath` is an execution contract. It is not the frontend `PatternPath`
type and it must not expose source-layer pattern semantics directly.

## Deterministic Path Reading

Current `AccessPath` rules:

- the root identifies the frame-local variable being accessed
- components are stored in source access order from root to leaf
- the empty component list means the whole variable root
- path encoding order is deterministic and stable

Example readings:

- `root` means the whole local variable
- `root[0]` means tuple child `TupleIndex(0)`
- `root[0][1]` means a nested tuple child path

## Overlap Rule

Two `AccessPath` values overlap if either path is a prefix of the other.

Current v0 overlap cases:

- exact: `root[0]` vs `root[0]`
- parent-child: `root[0]` vs `root[0][1]`
- child-parent: `root[0][1]` vs `root[0]`

Current v0 non-overlap case:

- siblings: `root[0]` vs `root[1]`

Overlap is root-sensitive:

- paths with different root `SymbolId` values do not overlap

## Borrow Lifetime v0

Current v0 lifetime rule:

- an active runtime borrow lives until frame exit

This means:

- borrow events add an `AccessPath` to the current frame-local active borrow set
- the active borrow set is cleared when that frame exits
- there is no earlier release mechanism in v0

This lifetime is intentionally narrower in implementation complexity than a
precise non-lexical release model.

## Supported Surface v0

The first admitted runtime ownership slice is tuple-only.

Current supported ownership surface:

- explicit tuple borrow capture
- tuple write targets
- overlap checks between tuple borrow paths and tuple write paths

Current unsupported ownership surface:

- record field paths
- ADT payload paths
- schema-derived paths
- sequence/index alias rules
- inter-frame borrow propagation

Unsupported cases must not be silently documented as if they are already part
of the runtime ownership contract.

## Crate Ownership Map

Current owner split:

- `sm-front`
  - source-level borrow capture semantics
  - source path extraction before lowering
- `sm-ir`
  - canonical lowered `AccessPath`
  - borrow/write path events in the execution IR
- `sm-emit`
  - SemCode transport and deterministic encoding for ownership path metadata
- `sm-verify`
  - structural validation of ownership payloads
- `sm-runtime-core`
  - shared execution-layer path vocabulary, if shared types are needed
- `sm-vm`
  - frame-local active borrow tracking
  - write-time overlap rejection

No execution-layer crate may depend back on frontend AST or parser internals to
evaluate this contract.

## Runtime Enforcement Rule

Once the v0 pipeline is fully landed, the VM must enforce this narrow rule:

- before a supported write executes, compare the write `AccessPath` against the
  current frame-local active borrowed paths
- if any borrowed path overlaps, reject the write with a deterministic runtime
  error
- if only sibling borrowed paths exist, permit the write

This rule is the only required runtime ownership behavior in v0.

## Verifier Rule

The verifier owns only structural checks for ownership metadata.

Current verifier responsibilities:

- validate ownership path payload shape
- validate component kinds admitted by the current SemCode version
- reject malformed or unsupported ownership payloads before execution

Current verifier non-goals:

- proving alias safety
- simulating borrow lifetime
- evaluating overlap policy during verification

## SemCode Compatibility Rule

Any ownership metadata added to SemCode is a binary-contract change.

Current compatibility rule:

- no silent mutation of SemCode format for ownership payloads
- ownership payload encoding must be documented explicitly
- version review and format notes are required in the same change series

## Explicit Non-Goals

This document does not define:

- a full runtime Rust-like ownership model
- region inference
- non-lexical lifetime release
- optimizer-driven alias reasoning
- ownership semantics for records, ADTs, or schemas in v0
- frontend-to-VM direct sharing of `PatternPath`

## Acceptance Reading

This contract is satisfied only when:

- explicit tuple borrow metadata survives frontend to IR to SemCode to VM
- runtime tracks active borrowed tuple paths per frame
- supported writes reject on exact, parent-child, and child-parent overlap
- sibling writes remain allowed
- malformed ownership payloads are rejected before execution
- unsupported ownership surfaces are not claimed as supported
