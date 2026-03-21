# Source Semantics Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current source-level execution meaning of Semantic
programs before lowering into IR, SemCode verification, or VM execution.

It complements `syntax.md` and `types.md` by specifying how the accepted
source forms are interpreted today.

It does not redefine:

- SemCode header or opcode rules
- verifier admission semantics
- VM runtime traps
- PROMETHEUS ABI effects

## Program Meaning

The current Rust-like executable surface is a deterministic function program.

Current rules:

- top-level executable items are functions only
- execution begins at `fn main()`
- `main` must currently have signature `fn main()`
- there is no dynamic entrypoint discovery or module-level executable code
- `fn name(...) -> ret = expr;` is semantically equivalent to a body containing
  only `return expr;`

## Deterministic Evaluation Order

Current source evaluation order is deterministic.

Current rules:

- binary expressions evaluate the left operand before the right operand
- call arguments are evaluated left-to-right
- control-flow conditions are evaluated before branch selection
- `match` scrutinee is evaluated once before arm dispatch
- pipeline stages evaluate left-to-right and pass the previous stage value as
  the first argument of the next call

The source contract does not currently claim short-circuit laziness beyond the
observable deterministic behavior of the current lowering path.

## Names And Call Resolution

Function-call resolution is lexical and deterministic.

Current rules:

- ordinary user-defined functions are resolved from the program function table
- builtin math calls are resolved only when no user-defined function of the
  same name exists
- there is no overload resolution
- there is no dynamic dispatch

Current builtin names in the Rust-like surface are:

- `sin`
- `cos`
- `tan`
- `sqrt`
- `abs`
- `pow`
- `assert`

## Scope And Binding Rules

The executable Rust-like surface uses lexical block scoping.

Current rules:

- function parameters are bound in function scope before body execution
- `let` introduces a source-visible local binding
- `if` branches and `match` arms are checked in branch-local scopes
- branch-local bindings do not escape to sibling branches

Current honest limit:

- the repository does not yet freeze a richer user-facing policy around
  same-scope rebinding or style-level shadowing; users should not depend on
  subtle local-name reuse behavior as a stable contract

## Statements

Current statement meaning:

- `let` evaluates the right-hand side before binding the name
- `name op= expr;` evaluates as read-modify-write over the existing binding
- the current v0 compound forms are `+=`, `-=`, `*=`, `/=`, `&&=`, and `||=`
- `guard condition else return ...;` continues when the condition is `true`
- when the guard condition is `false`, the `else return` path terminates the
  current function immediately
- `assert(condition);` continues when `condition` is `true`
- `assert(condition);` terminates through the core fail-fast trap path when
  `condition` is `false`
- expression statements evaluate for effect and then discard any produced value
- `return expr;` terminates the current function with that value
- `return;` terminates a `unit`-returning function

Current non-goal:

- the source contract does not claim deferred execution, generators, or
  coroutine-style statement behavior
- `guard` does not yet support arbitrary `else { ... }` recovery blocks
- plain reassignment `name = expr;` is not yet part of the public surface

## Block Expressions

Current block-expression semantics:

- `{ ... tail }` evaluates its body in a fresh lexical block scope
- body statements run in source order before the tail expression
- the final unterminated tail expression becomes the value of the block
- block-local bindings do not escape the block expression

Current v0 limit:

- block-expression bodies currently accept only `let` bindings and expression
  statements before the tail value
- `return` is not yet supported inside value-producing block bodies as a
  stable source contract

## Control Flow

### If

Current `if` semantics:

- `if` requires a `bool` condition
- the `then` branch runs when the condition is `true`
- otherwise the `else` branch runs
- `else if` is treated as nested `if` inside the `else` branch

Current `if` expression semantics:

- `if condition { ... } else { ... }` may appear in value position
- both branches are evaluated through value-producing block semantics
- both branches must produce the same type
- `else` is required for value-producing `if`

`quad` is intentionally not treated as an implicit condition type. Users must
write explicit comparisons.

Current v0 limit:

- `else if` sugar is not yet supported for value-producing `if`; users must
  write `else { if ... }`

### Guard

Current `guard` semantics:

- `guard condition else return;` is allowed in `unit`-returning functions
- `guard condition else return expr;` is allowed when `expr` matches the
  function return type
- `guard` requires a `bool` condition
- `guard` is purely statement-level in the current source contract

### Match

Current `match` semantics:

- `match` is currently restricted to `quad`
- arms match only the literal patterns `N`, `F`, `T`, `S`
- non-default arms may attach a `bool` guard with `if guard_expr`
- `_` is required as the default arm
- the first matching arm is selected deterministically

Current `match` expression semantics:

- `match scrutinee { ... }` may appear in value position
- each non-default arm uses a value-producing block after `=>`
- expression arms may use the same `if guard_expr` form as statement-side arms
- `_` is required as the default arm for value-producing `match`
- all arms, including `_`, must produce the same type

This is a deliberately narrow source contract rather than a full general
pattern-matching system.

Current v0 limit:

- the default `_` arm does not yet support guards
- only literal `quad` patterns and `_` are part of the stable source contract

## Operator Meaning

Current operator meaning:

- `==` and `!=` produce `bool`
- `&&` and `||` work on `bool` and `quad` only when both operands are of the
  same family
- `!` works on `bool` and `quad`
- `->` is quad implication and returns `quad`
- `+`, `-`, `*`, `/` currently have stable arithmetic meaning only on `f64`

Current honest limit:

- `fx` value flow is supported, but `fx` arithmetic is intentionally narrower
  than `f64` arithmetic in the Rust-like source surface

## Builtin Call Meaning

Builtin math calls are part of the source contract, not a foreign escape hatch.

Current rules:

- builtins are type-checked before lowering
- builtin math calls lower through the same call surface as ordinary functions
- `assert` shares ordinary call-like syntax but lowers to a dedicated core
  assertion opcode rather than an ordinary function call
- the verified execution path recognizes the supported builtin set explicitly

Current builtin signatures:

- `sin(f64) -> f64`
- `cos(f64) -> f64`
- `tan(f64) -> f64`
- `sqrt(f64) -> f64`
- `abs(f64) -> f64`
- `pow(f64, f64) -> f64`
- `assert(bool);` as a statement-level builtin contract

## Pipeline

Current `|>` semantics:

- `input |> stage()` is equivalent to `stage(input)`
- `input |> stage(arg1, arg2)` is equivalent to `stage(input, arg1, arg2)`
- pipeline stages are currently restricted to bare function names or ordinary
  call syntax

Current v0 limit:

- placeholder-based pipeline forms are not part of the current contract
- arbitrary right-hand expressions after `|>` are not yet supported

## Logos Semantics Boundary

The Logos-oriented surface is declarative rather than executable in the same
way as the Rust-like function path.

Current rules:

- Logos programs describe `System`, `Entity`, and `Law` declarations
- law ordering is deterministic by descending priority
- current `When` condition/effect bodies are stored as structured text
  fragments at this stage
- Logos input does not lower directly into the Rust-like SemCode function path

## Non-Goals

The current source semantics contract does not yet claim stable support for:

- exceptions
- heap/object semantics
- async or concurrency execution
- user-defined operator overloading
- lazy evaluation as a first-class source feature
- dynamic imports or runtime package resolution

## Contract Rule

Any public change to source-level call resolution, block scope behavior,
control-flow meaning, operator meaning, or builtin semantics should update this
document in the same change series.
