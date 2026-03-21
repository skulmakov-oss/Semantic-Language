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

Current rules:

- `fn` introduces a function
- parameters are named and typed explicitly
- the return type is optional; omitted return type means `unit`
- function bodies are block-delimited with `{ ... }`
- the public program entrypoint is `fn main()`

## Statements

Current statement forms:

- `let name = expr;`
- `let name: type = expr;`
- `if condition { ... } else { ... }`
- `match quad_expr { T => { ... } ... _ => { ... } }`
- `return;`
- `return expr;`
- expression statements: `expr;`

Current statement rules:

- semicolons terminate executable statements
- `if` conditions must be `bool`
- `match` is currently restricted to `quad`
- `match` requires an explicit default arm `_ => { ... }`
- unit-returning calls may be used as statements

## Expressions

Current expression forms:

- literals:
  - quad literals: `N`, `F`, `T`, `S`
  - bool literals: `true`, `false`
  - integer literals
  - floating literals
- variables
- function calls
- block expressions with a trailing tail value:
  - `{ let x = 1; x }`
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

Current precedence, from tighter to looser:

1. primary expressions and calls
2. unary `!`, unary `+`, unary `-`
3. `*`, `/`
4. `+`, `-`
5. `==`, `!=`
6. `&&`
7. `||`
8. `->`

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
```

Current builtins are resolved as part of the public source surface and must not
require a separate foreign-call syntax.

## Imports And Module Surface

The current repository supports source-level imports and re-exports. That
surface is part of the language contract and is specified in `modules.md`.

## Current Exclusions

The current source contract does not yet claim stable support for:

- `if` or `match` as value-producing expressions
- relational operators such as `>`, `<`, `>=`, `<=`
- user-defined aggregate types
- collections as first-class language forms
- generics or trait-like abstraction
- exceptions or Python-style dynamic execution
- concurrency-oriented source constructs

## Contract Rule

Any public change to source syntax, source statement forms, or operator meaning
should update this document in the same change series.
