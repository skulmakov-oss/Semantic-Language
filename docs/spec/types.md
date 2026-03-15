# Source Type Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current public source-level type contract for
Semantic programs.

It covers the executable source surface rather than the SemCode or VM
representation layer.

## Current Type Family

Current source-visible types:

- `quad`
- `bool`
- `i32`
- `u32`
- `f64`
- `fx`
- `unit`
- `qvec(N)` as a reserved parser-level family

## Unit

`unit` is the implicit type of functions without an explicit return type.

Current rule:

- `fn main()` must currently have implicit `unit` return and no parameters in
  the canonical executable path

## Quad

`quad` is a first-class semantic logic type with four values:

- `N`
- `F`
- `T`
- `S`

Current rules:

- `quad` participates in equality and implication
- `match` currently operates only on `quad`
- `quad` is not accepted directly as an `if` condition

## Bool

`bool` is the ordinary binary condition type.

Current rules:

- `if` conditions must evaluate to `bool`
- `!`, `&&`, and `||` are valid on `bool`
- equality comparisons on `bool` are valid

## I32 And U32

`i32` and `u32` are the current integer-oriented execution types.

Current rules:

- arithmetic operators are expected to stay within the same numeric family
- equality comparisons are valid inside the same family
- implicit cross-family numeric coercion is not part of the current contract

## F64

`f64` is the current floating-point math family.

Current rules:

- `f64` availability is gated by the active parser profile / compile policy
- arithmetic operators `+`, `-`, `*`, `/` are supported on `f64`
- equality comparisons are supported on `f64`
- current builtin math calls include:
  - `sin`
  - `cos`
  - `tan`
  - `sqrt`
  - `abs`
  - `pow`

## Fx

`fx` is the fixed-point-oriented numeric family.

Current rules:

- the canonical `fx` value path is end-to-end
- explicit `fx` annotations are supported
- `fx` currently accepts literals and existing `fx`-typed values in the public
  Rust-like path

Current honest limits:

- richer `fx` arithmetic remains narrower than the `f64` surface
- coercion from non-literal non-`fx` expressions is not yet the full intended
  long-term contract
- unary `+` and unary `-` on `fx` are still intentionally limited in the
  canonical Rust-like path

## QVec

`qvec(N)` exists as a parser-level family and should be treated as reserved or
partial rather than fully stabilized in the current public source contract.

Until the repository documents a fuller execution and library story for
`qvec(N)`, it should not be treated as a broadly stable user-facing type family.

## Equality And Control Rules

Current equality and control rules:

- `==` and `!=` require meaningful same-family comparisons
- `if` requires `bool`
- `match` requires `quad`

## Function Typing Rules

Current function typing rules:

- every parameter has an explicit type
- return type defaults to `unit` if omitted
- `return expr;` must match the declared return type
- `return;` is valid only for `unit`-returning functions

## Builtin Typing Rule

Builtin functions are part of the public type contract and are checked as typed
calls, not as dynamically typed escape hatches.

That means:

- argument count must match
- argument types must match the builtin signature
- builtin typing failures are ordinary source-level type errors

## Current Exclusions

The current source type contract does not yet claim stable support for:

- user-defined aggregate types
- algebraic data types
- generics
- trait or protocol systems
- implicit numeric widening across unrelated families
- a broad collection type ecosystem

## Contract Rule

Any public change to source-visible type meaning or source type-checking rules
should update this document in the same change series.
