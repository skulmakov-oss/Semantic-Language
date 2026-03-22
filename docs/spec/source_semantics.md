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
- short lambda call-site sugar evaluates its argument before the lambda body
  binding becomes visible

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
- named arguments reorder only after the target function resolves successfully

Current builtin names in the Rust-like surface are:

- `sin`
- `cos`
- `tan`
- `sqrt`
- `abs`
- `pow`
- `assert`

Current named-argument call semantics:

- named arguments are currently supported only for ordinary user-defined
  functions
- positional arguments may appear only before the first named argument
- after resolution, named arguments reorder to the declared parameter order
  before ordinary argument type-checking and lowering
- each required non-default parameter must receive exactly one argument in the
  current contract

Current default-parameter limits:

- builtin calls do not yet accept named arguments
- named arguments do not imply overload resolution or keyword-only parameters

Current default-parameter semantics:

- default parameters are currently supported only for ordinary user-defined
  functions
- only trailing parameters may declare defaults in the stable v0 surface
- when an argument is omitted for a trailing defaulted parameter, the declared
  default initializer is substituted before ordinary argument type-checking and
  lowering
- default initializers must remain within the current const-safe expression
  subset and may not depend on function parameters or local runtime bindings

Current v0 limits:

- builtin calls do not yet expose default parameters
- default parameters do not imply optional positional holes, keyword-only
  parameters, or overload resolution

## Numeric Literal Meaning

Current numeric-literal rules:

- unsuffixed decimal integer literals currently evaluate as `i32`
- unsuffixed decimal literals containing `.` currently evaluate as `f64`
- explicit `i32`, `u32`, `f64`, and `fx` suffixes fix the source-level literal type
- digit separators `_` are ignored for numeric-literal value decoding
- hexadecimal literals currently decode only for integer carriers
- explicit `fx` literals lower directly to the fixed-point value carrier and do
  not rely on `f64` surface policy

Current honest limit:

- extended numeric literal spelling does not currently widen integer arithmetic
  beyond the already documented operator surface
- exponent notation and binary/octal literal families are not yet part of the
  stable contract

## Range Literal

Current range-literal semantics:

- `start..end` denotes a half-open ascending `i32` interval descriptor
- `start..=end` denotes a closed ascending `i32` interval descriptor
- both bounds are evaluated exactly once, left to right
- the current lowering path reuses an internal tuple-shaped carrier rather than
  introducing a dedicated runtime range family

Current v0 limit:

- range literals currently require `i32` bounds
- range literals are not yet part of the stable tuple/user-data surface
- range equality is not yet part of the stable source contract
- `for ... in range` is not yet part of the stable source contract
- the source contract does not yet treat ranges as a general iterable family

## Scope And Binding Rules

The executable Rust-like surface uses lexical block scoping.

Current rules:

- function parameters are bound in function scope before body execution
- `const` introduces an immutable source-visible local binding
- `let` introduces a source-visible local binding
- `let (a, b) = expr;` evaluates the right-hand side once and then binds named
  items left-to-right from the produced tuple value
- `let (a, T) = expr else return ...;` evaluates the right-hand side once,
  projects flat tuple items, and returns immediately when any refutable
  quad-literal item does not match
- `let _ = expr;` evaluates the right-hand side but introduces no source-visible binding
- `if` branches and `match` arms are checked in branch-local scopes
- branch-local bindings do not escape to sibling branches

Current honest limit:

- the repository does not yet freeze a richer user-facing policy around
  same-scope rebinding or style-level shadowing; users should not depend on
  subtle local-name reuse behavior as a stable contract

## Statements

Current statement meaning:

- `const` evaluates a compile-time-safe initializer expression before binding the name
- const bindings are immutable in the current source contract
- `let` evaluates the right-hand side before binding the name
- tuple destructuring bind evaluates the right-hand side once before projecting
  tuple items into named bindings
- tuple destructuring assignment evaluates the right-hand side once before
  projecting tuple items into existing assignment targets
- tuple `let-else` introduces named bindings only on the success path; failure
  follows the explicit `else return` path
- discard bind evaluates the right-hand side and then drops the produced value
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

Current v0 const limit:

- `const` is statement-level only in the current source contract
- const initializers currently support only pure literal/const expression forms
- ordinary function calls, control-flow expressions, and references to
  non-const locals are not yet part of the stable const initializer subset

## Block Expressions

Current block-expression semantics:

- `{ ... tail }` evaluates its body in a fresh lexical block scope
- body statements run in source order before the tail expression
- the final unterminated tail expression becomes the value of the block
- block-local bindings do not escape the block expression

Current v0 limit:

- block-expression bodies currently accept only named `let` bindings,
  tuple destructuring binds, `const` bindings, discard binds, and expression
  statements before the tail value
- tuple `let-else` is not yet part of the stable value-producing block-body
  contract
- discard bind is accepted in value-producing block bodies, but richer pattern
  destructuring is not yet part of the stable block-expression contract
- `return` is not yet supported inside value-producing block bodies as a
  stable source contract

## Where-Clause

Current `where` semantics:

- `expr where a = x, b = y` is expression-suffix sugar over an ordinary
  value-producing block
- `where` bindings execute in source order before the tail expression
- later `where` bindings may reference earlier `where` bindings
- the tail expression sees all `where` bindings introduced by the clause

Current v0 limit:

- `where` is currently expression-suffix sugar only
- `where` bindings currently reuse ordinary local-bind semantics; richer
  destructuring and control-flow forms are not yet part of the stable contract

## UFCS / Method-Call Sugar

Current UFCS semantics:

- `receiver.name(args...)` is postfix sugar over the ordinary call form
  `name(receiver, args...)`
- after desugaring, ordinary parameter ordering, named-argument reordering, and
  lowering rules apply unchanged
- UFCS chaining remains ordinary nested-call structure after desugaring

Current v0 limit:

- UFCS currently requires explicit call parentheses
- UFCS does not define field access or member lookup
- UFCS does not introduce method declarations or object-oriented dispatch

## Loop Expression

Current `loop` expression semantics:

- `loop { ... }` produces a value through explicit `break expr;`
- `break` values inside the same loop must agree on one result type
- lowering uses the existing label/jump/store/load path; no separate runtime
  carrier is introduced for this slice

Current v0 limit:

- only expression-form `loop` is part of the contract
- only `break expr;` is supported; bare `break;` is not
- loop-expression bodies currently do not allow `let-else`, `guard`, or
  `return`
- `continue`, statement-loop, and richer control interaction are deferred

## Tuple Destructuring Bind

Current tuple-destructuring semantics:

- `let (a, b) = expr;` requires the right-hand side to produce a tuple value
- the tuple arity must match the binding item count exactly
- `_` items discard the corresponding tuple element without introducing a
  source-visible binding
- an optional whole-binding annotation, such as
  `let (a, b): (i32, bool) = expr;`, constrains the full tuple value before
  item bindings are introduced
- `(a, b) = expr;` assigns projected tuple items into already existing mutable
  bindings
- `_` items in tuple destructuring assignment discard the corresponding tuple
  element without requiring a target binding

Current v0 limit:

- tuple destructuring bind is currently statement-level only
- tuple `let-else` is currently statement-level only
- tuple destructuring assignment is currently statement-level only
- only flat name-or-`_` item lists are supported for plain tuple destructuring
- tuple `let-else` currently supports only flat name/`_`/quad-literal item
  lists
- tuple destructuring assignment does not introduce new bindings; every named
  item must already resolve to an existing non-const local
- nested tuple patterns, tuple field access, and general tuple pattern matching
  are not yet part of the stable source contract
- plain `let name = expr else ...` is not yet part of the stable source
  contract

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

## Tuples

Current tuple semantics:

- tuple literals evaluate their elements left-to-right and package them into a
  single aggregate value
- tuple types are structural and preserve declared element order
- tuples may flow through ordinary local bindings, function parameters, returns,
  equality, and the verified execution path

Current v0 limit:

- tuple literals and tuple types require arity at least 2
- tuples are currently value carriers only; destructuring, field access, and
  tuple-specific operators are not yet part of the stable contract
- tuple values are not part of the PROMETHEUS host ABI surface

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
- `_` currently means wildcard/default only in `match`, not a general rich
  pattern system
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

## Short Lambdas

Current short-lambda semantics:

- short lambdas are currently capture-free call-site sugar only
- `(x => expr)(arg)` is interpreted as a fresh lexical block equivalent to
  `{ let x = arg; expr }`
- `value |> (x => expr)` is interpreted as the same block sugar with `value` as
  the bound argument
- the lambda body is checked and lowered through ordinary block-expression
  semantics; no alternate runtime callable representation is introduced

Current v0 limits:

- short lambdas are not first-class values
- short lambdas currently support exactly one parameter and exactly one applied
  argument
- outer local-name capture is rejected in the current source contract

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
- `input |> stage(name = arg1)` is equivalent to `stage(input, name = arg1)`
- `input |> stage(name = arg1)` may still omit later trailing defaulted
  parameters
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
