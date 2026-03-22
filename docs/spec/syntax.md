# Source Syntax Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current canonical source-level syntax for the public
Semantic language surface.

It describes the source language accepted by the Rust-like frontend path. It
does not redefine:

- SemCode binary structure
- verifier admission rules
- VM runtime behavior
- PROMETHEUS ABI semantics

## Current Source Surfaces

Semantic currently exposes two distinct source-oriented surfaces:

- a Rust-like executable surface used for functions, expressions, and control
  flow
- a Logos-oriented declarative surface used for `System`, `Entity`, and `Law`
  forms

This document covers the Rust-like executable surface first. The remaining
public source-surface contracts are specified separately in:

- `source_semantics.md`
- `diagnostics.md`
- `modules.md`
- `logos.md`
- `docs/LANGUAGE.md`

## Program Structure

The current Rust-like executable program is a sequence of top-level functions:

```sm
fn name(arg: type, ...) -> ret_type {
    ...
}
```

Expression-bodied sugar is also part of the current v0 surface:

```sm
fn name(arg: type, ...) -> ret_type = expr;
```

Current rules:

- `fn` introduces a function
- parameters are named and typed explicitly
- the return type is optional; omitted return type means `unit`
- function bodies are block-delimited with `{ ... }`
- `fn ... = expr;` is accepted as shorthand for a single returned expression
- the public program entrypoint is `fn main()`

## Statements

Current statement forms:

- `const name = expr;`
- `const name: type = expr;`
- `let name = expr;`
- `let name: type = expr;`
- `let _ = expr;`
- `let _: type = expr;`
- `name += expr;`
- `name -= expr;`
- `name *= expr;`
- `name /= expr;`
- `name &&= expr;`
- `name ||= expr;`
- `guard condition else return;`
- `guard condition else return expr;`
- `assert(condition);`
- `if condition { ... } else { ... }`
- `match quad_expr { T => { ... } ... _ => { ... } }`
- `match quad_expr { T if ready == true => { ... } ... _ => { ... } }`
- `return;`
- `return expr;`
- expression statements: `expr;`

Current statement rules:

- semicolons terminate executable statements
- `const` is currently statement-level only
- `const` initializer syntax mirrors `let` but uses a narrower compile-time-safe expression subset
- `let _ = expr;` is the current discard-bind surface
- compound assignment is statement-level sugar only
- `guard` currently supports only the `else return` form
- `assert(condition);` is a statement-level builtin contract form
- `if` conditions must be `bool`
- `match` is currently restricted to `quad`
- `match` requires an explicit default arm `_ => { ... }`
- `_` in `match` remains the current wildcard/default arm spelling
- unit-returning calls may be used as statements
- extended numeric literal spelling does not itself widen arithmetic support

## Expressions

Current expression forms:

- literals:
  - quad literals: `N`, `F`, `T`, `S`
  - bool literals: `true`, `false`
  - integer literals:
    - decimal `123`
    - decimal with separators `1_000`
    - hexadecimal `0xff`
    - explicit typed forms `123i32`, `123u32`, `0xffu32`
  - floating and fixed-point literals:
    - decimal `1.25`
    - decimal with separators `1_000.25`
    - explicit `f64` forms `1.25f64`, `100f64`
    - explicit `fx` forms `1.25fx`, `100fx`
- variables
- function calls
- named-argument calls:
  - `open(path = main_path, mode = read_only)`
  - `value |> stage(limit = 10)`
- pipeline chains:
  - `value |> stage()`
  - `value |> stage(arg)`
  - `value |> (x => expr)`
- short lambda immediate-call sugar:
  - `(x => expr)(arg)`
- block expressions with a trailing tail value:
  - `{ let x = 1; x }`
- `if` expressions with explicit `else` blocks:
  - `if ready { 1.0 } else { 0.0 }`
- `match` expressions with value-producing arms:
  - `match state { T => { 1.0 } _ => { 0.0 } }`
  - `match state { T if ready == true => { 1.0 } _ => { 0.0 } }`
- parenthesized expressions
- unary operators:
  - `!`
  - unary `+`
  - unary `-`
- binary operators:
  - `*`, `/`
  - `+`, `-`
  - `==`, `!=`
  - `&&`, `||`
  - `->`

Current v0 numeric-literal limits:

- unsuffixed integer literals currently mean `i32`
- unsuffixed decimal literals with `.` currently mean `f64`
- explicit `fx` literals are decimal-only and do not require `f64` surface policy
- hexadecimal literals currently target only integer carriers
- exponent notation and binary/octal literal families are not yet part of the
  stable surface
- typed literal spelling does not imply new integer arithmetic beyond the
  already documented operator surface

Current precedence, from tighter to looser:

1. primary expressions and calls
2. unary `!`, unary `+`, unary `-`
3. `*`, `/`
4. `+`, `-`
5. `==`, `!=`
6. `&&`
7. `||`
8. `->`
9. `|>`

Current short-lambda rules:

- short lambda syntax is currently single-parameter only: `x => expr`
- short lambdas are not first-class values in v0
- the stable v0 surface accepts short lambdas only as:
  - immediate call sugar: `(x => expr)(arg)`
  - pipeline stage sugar: `value |> (x => expr)`
- short lambdas are capture-free in v0; they may not reference outer local
  bindings
- typed lambda parameters and multi-argument lambda forms are not yet part of
  the stable source contract

Current named-argument rules:

- ordinary user-defined calls may use named arguments
- positional arguments are allowed only as a leading prefix before any named
  argument
- every declared parameter must still be supplied exactly once in v0
- named arguments reorder to the declared parameter order before ordinary
  type-checking and lowering
- named arguments are not yet part of the builtin-call surface

## Quad-Specific Surface Rules

Current `quad` model values are:

- `N` = unknown
- `F` = false
- `T` = true
- `S` = conflict

Current source restrictions:

- `if quad_expr` is forbidden; users must write an explicit comparison
- `match` currently accepts only `quad` scrutinees
- quad implication uses `->`

## Builtin Calls

Builtin calls currently share call syntax with ordinary functions:

```sm
sqrt(9.0)
pow(2.0, 3.0)
abs(-1.0)
assert(ready == true);
```

Current builtins are resolved as part of the public source surface and must not
require a separate foreign-call syntax. `assert` shares ordinary call-like
syntax but is statement-only in the current contract.

## Imports And Module Surface

The current repository supports source-level imports and re-exports. That
surface is part of the language contract and is specified in `modules.md`.

## Current Exclusions

The current source contract does not yet claim stable support for:

- relational operators such as `>`, `<`, `>=`, `<=`
- user-defined aggregate types
- collections as first-class language forms
- generics or trait-like abstraction
- exceptions or Python-style dynamic execution
- concurrency-oriented source constructs

## Contract Rule

Any public change to source syntax, source statement forms, or operator meaning
should update this document in the same change series.
