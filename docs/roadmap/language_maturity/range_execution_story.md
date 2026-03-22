# Range Execution Story

Status: proposed v0

## Purpose

This document defines the narrow executable contract that must exist before
Semantic can honestly stabilize:

- `D20-B01` range literals
- `D20-B02` `for` over range

It is a design target for the language-density roadmap, not a claim that the
current parser, IR, verifier, or VM already expose this surface.

## Why This Needs A Story First

Ranges look syntactically cheap, but they are not a "surface-only" feature in
the current language.

Today Semantic has:

- expression and control-flow sugar for blocks, `if`, `match`, `loop`, and
  `where`
- integer literal carriers `i32` and `u32`
- equality operators and compound assignment

But it does not yet have:

- public source-level relational operators such as `<`, `<=`, `>`, `>=`
- a stabilized iterable protocol
- a stabilized first-class range value family
- a documented loop-bound execution contract for integer intervals

Without an explicit executable story, `a..b` would be a paper-only syntax
token. That would be a bad PR.

## Proposed First-Wave Scope

The first-wave range surface should stay narrow.

Proposed source forms:

```sm
start..end
start..=end
for i in start..end { ... }
for i in start..=end { ... }
```

## Type Scope

The first stable range family should be `i32`-only.

Why:

- unsuffixed integer literals already normalize to `i32`
- `i32` is the simplest first loop-carrier family to document honestly
- mixed signedness and `u32` loop semantics would widen the contract too early

Stage-1 rules:

- both bounds must type-check as `i32`
- mixed bound types are rejected
- range literals do not imply general integer arithmetic beyond the already
  documented operator surface

## Value Model Boundary

The first stabilized range surface should not pretend to be a general
collection.

Stage-1 rules:

- ranges are executable interval descriptors, not list values
- the only stable consumer in the first wave is `for i in ...`
- passing a range as a function argument, returning it, storing it in tuples,
  or comparing ranges is out of scope
- there is no iterator protocol, `next`, or step-size customization in the
  first wave

This keeps `D20-B01` and `D20-B02` honest: they are about deterministic loop
syntax, not about introducing a whole iterable subsystem.

## Bounds Semantics

Stage-1 intervals are ascending integer intervals with deterministic empty-loop
behavior.

Proposed rules:

- `start..end` is half-open: it visits `start, start + 1, ...` and stops before
  `end`
- `start..=end` is closed: it visits `start, start + 1, ...` and includes `end`
- if `start >= end`, `start..end` produces zero iterations
- if `start > end`, `start..=end` produces zero iterations
- if `start == end`, `start..=end` produces exactly one iteration

This defines user-visible behavior without requiring the general relational
operator family to become part of the public source surface in the same change.

## Evaluation Order

Bounds must be evaluated exactly once, left to right.

Proposed rules:

- evaluate `start` once
- evaluate `end` once
- store both into hidden loop-carrier locals before the first iteration test
- do not re-evaluate bound expressions per iteration

This preserves deterministic side-effect behavior and keeps quota accounting
predictable.

## Lowering Direction

The preferred first implementation is not a generic iterator model.

It should lower `for i in start..end { body }` into a canonical counted-loop
shape:

1. evaluate and store hidden `current`
2. evaluate and store hidden `end`
3. perform a narrow integer bound test
4. on success, bind `i` for the current iteration
5. execute the loop body
6. increment `current` by one
7. jump back to the loop-bound test

Inclusive form follows the same pattern with a closed-interval test.

The important contract point is not the exact local names or opcode spelling.
It is that the lowering is:

- deterministic
- bounded
- transparent
- free of host interaction

## IR And VM Implications

The first range family may require narrow internal support for integer
loop-bound testing.

Preferred rule:

- if IR or VM additions are needed, keep them specific to integer loop bounds
  rather than using range syntax as a back door for a broad relational-operator
  expansion

Acceptable first-wave implementation directions include:

- dedicated narrow compare/test support for `i32` loop bounds
- a verifier-visible loop-bound helper path that remains internal to the range
  lowering contract

Non-acceptable direction:

- introducing an open-ended dynamic iterator or host callback model

## Quotas And Determinism

`for` over range must preserve the existing verified execution model.

Required rules:

- iteration count must remain visible to ordinary quota accounting
- there is no hidden host iterator state
- replay and trace surfaces must observe the same deterministic interval
  behavior

## Boundary With Source Relational Operators

The first range wave must not implicitly claim that general source-level `<`,
`<=`, `>`, `>=` are stabilized.

Explicit rule:

- range execution may use a narrow internal loop-bound comparison strategy
  without making relational operators part of the public source syntax contract

This keeps the change series honest and prevents a cheap-looking loop feature
from silently widening the whole expression language.

## Non-Goals

This first range wave does not attempt to provide:

- descending ranges
- custom step sizes
- `u32`, `f64`, or `fx` ranges
- range values as general user data
- range pattern matching
- collection iteration
- iterator traits or protocols
- implicit widening of source-level relational operators
- PROMETHEUS ABI, capability, runtime-state, or rule-engine changes

## Acceptance Criteria

The repository should consider the range story properly fixed before `#84` and
`#85` land when:

- the first stable range carrier is explicit
- empty-loop and inclusive/exclusive semantics are explicit
- bound evaluation order is explicit
- the lowering direction is explicit
- the boundary with general relational operators is explicit
- the repository has an honest statement that ranges are loop-oriented
  executable interval descriptors, not a hidden iterable subsystem
