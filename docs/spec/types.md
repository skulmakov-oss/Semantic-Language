# Source Type Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current public source-level type contract for
Semantic programs.

It covers the executable source surface rather than the SemCode or VM
representation layer.

Compile-time-only declaration families such as `schema Name { ... }` are part
of the source contract, but they are not yet executable source-visible types or
VM value families.

Operational source-level meaning such as call resolution, control-flow
selection, and source diagnostics is specified separately in:

- `source_semantics.md`
- `diagnostics.md`

## Current Type Family

Current source-visible types:

- `quad`
- `bool`
- `i32`
- `u32`
- `f64`
- `fx`
- measured numeric forms such as `f64[m]` and `u32[ms]`
- `unit`
- `qvec(N)` as a reserved parser-level family

Current compile-time-only declaration families:

- nominal `schema Name { ... }` declarations for boundary/model contracts
- record-shaped and tagged-union schema forms within that compile-time-only
  declaration family

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
- `match` currently operates on `quad`, nominal enum scrutinees, and the
  standard-form `Option(T)` / `Result(T, E)` families
- `quad` is not accepted directly as an `if` condition

## Standard Forms

Current first-wave standard forms:

- `Option(T)` is the canonical optional-value type family in declared type
  positions
- `Result(T, E)` is the canonical success/error type family in declared type
  positions
- these forms are language-owned standard families, not user-defined generic
  declarations
- they currently lower through the same canonical aggregate carrier family used
  by nominal ADTs
- explicit `Option::Some/None` and `Result::Ok/Err` patterns participate in
  the stable match surface over these families

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

## Units Of Measure

First-wave units of measure are source-level refinements over the existing core
numeric families.

Current supported forms:

- `i32[unit]`
- `u32[unit]`
- `f64[unit]`
- `fx[unit]`

Current rules:

- the bracket payload is a single unit symbol
- measured numeric types may appear in locals, parameters, returns, tuple
  elements, record fields, `Option(T)`, and `Result(T, E)` payload positions
- assignment, call, return, and pattern-binding transport require exact base
  type and exact unit-symbol equality
- `+`, `-`, `==`, and `!=` are valid only when both operands have the same
  measured type
- lowering erases the unit annotation after semantic validation and reuses the
  existing numeric execution carrier

Current honest limits:

- units are not part of the VM value representation or public host ABI shape
- implicit conversions between unit symbols are not part of the contract
- compound unit algebra such as `m/s`, `N*m`, or exponent notation is not part
  of the first-wave surface
- `*` and `/` on unit-carrying values are intentionally rejected in the
  first-wave contract

## QVec

`qvec(N)` exists as a parser-level family and should be treated as reserved or
partial rather than fully stabilized in the current public source contract.

Until the repository documents a fuller execution and library story for
`qvec(N)`, it should not be treated as a broadly stable user-facing type family.

## Equality And Control Rules

Current equality and control rules:

- `==` and `!=` require meaningful same-family comparisons
- `if` requires `bool`
- `match` requires `quad`, nominal enum, `Option(T)`, or `Result(T, E)`

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

- schema names as executable local, parameter, or return types
- user-defined parameterized algebraic data types
- generics
- trait or protocol systems
- implicit numeric widening across unrelated families
- a broad collection type ecosystem

## Contract Rule

Any public change to source-visible type meaning or source type-checking rules
should update this document in the same change series.
