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

The current Rust-like executable program is a sequence of top-level `record`
declarations and functions:

```sm
record Name {
    field: type,
    ...
}
```

```sm
record DecisionContext {
    camera: quad,
    quality: f64,
}
```

Function surface remains:

```sm
fn name(arg: type, ...) -> ret_type {
    ...
}
```

Trailing default-parameter sugar is also part of the current v0 surface:

```sm
fn name(arg: type, optional_arg: type = expr) -> ret_type {
    ...
}
```

First-wave declaration contracts are also part of the current surface:

```sm
fn name(arg: type, ...) -> ret_type requires(condition) {
    ...
}
```

```sm
fn name(arg: type, ...) -> ret_type ensures(condition) {
    ...
}
```

Expression-bodied sugar is also part of the current v0 surface:

```sm
fn name(arg: type, ...) -> ret_type = expr;
```

Current rules:

- `record` introduces a nominal top-level record declaration
- record declarations must be non-empty
- record field names must be unique within one declaration
- record field types currently use the ordinary source type grammar
- `fn` introduces a function
- parameters are named and typed explicitly
- trailing parameters may attach a default initializer with `= expr`
- zero or more `requires(condition)` clauses may appear after the signature and
  before the function body
- zero or more `ensures(condition)` clauses may appear after any `requires`
  clauses and before the function body
- the return type is optional; omitted return type means `unit`
- function bodies are block-delimited with `{ ... }`
- `fn ... = expr;` is accepted as shorthand for a single returned expression
- the public program entrypoint is `fn main()`

Current v0 record limits:

- `RecordName { field: expr, ... }` is the current stage-1 record construction form
- `record_value.field_name` is the current stage-1 read-only field access form
- `record_value with { field: expr, ... }` is the current stage-2 immutable record update form
- record literal fields must appear exactly once by name
- lowering preserves declaration-slot order rather than source-field order
- record types may now appear in executable local bindings, parameters, and returns
- record destructuring and record copy-with now participate in the stable source contract
- record punning now participates in the stable source contract only as field shorthand inside canonical nominal record forms
- record equality is allowed only when every field type already supports stable equality
- record values are not part of the PROMETHEUS host ABI surface
- anonymous brace-only record literals/patterns, mutation, methods, and inheritance are not part of this slice

## Statements

Current statement forms:

- `const name = expr;`
- `const name: type = expr;`
- `let name = expr;`
- `let name: type = expr;`
- `let (a, b) = expr;`
- `let (a, _): (type_a, type_b) = expr;`
- `let RecordName { field_name: local_name, other_field: _ } = expr;`
- `let RecordName { field_name: T, other_field: local_name } = expr else return;`
- `let next = current with { quality: 1.0 };`
- `let (a, T) = expr else return;`
- `let (a, T): (type_a, quad) = expr else return expr;`
- `let _ = expr;`
- `let _: type = expr;`
- `name += expr;`
- `name -= expr;`
- `name *= expr;`
- `name /= expr;`
- `name &&= expr;`
- `name ||= expr;`
- `(a, b) = expr;`
- `for name in 0..10 { ... }`
- `for name in 0..=10 { ... }`
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
- tuple destructuring bind is currently flat only and accepts only names or `_`
- record destructuring bind is currently statement-level only
- record destructuring bind uses `RecordName { field: target }` and now also allows field shorthand `RecordName { field }`
- record destructuring bind currently supports only named targets or `_`
- record `let-else` is currently statement-level only
- record `let-else` uses `RecordName { field: target } = expr else return ...;` and also allows shorthand bind items `RecordName { field }`
- record `let-else` currently allows refutable items only through explicit `quad` literals `N/F/T/S`
- plain record destructuring bind does not currently accept quad-literal field targets
- tuple `let-else` currently requires tuple destructuring target and `else return`
- `let-else` tuple items are currently flat only and accept only names, `_`,
  or `quad` literals `N/F/T/S`
- tuple destructuring assignment is currently flat only and accepts only names or `_`
- compound assignment is statement-level sugar only
- `for ... in range` currently accepts only `i32` range expressions
- `guard` currently supports only the `else return` form
- `assert(condition);` is a statement-level builtin contract form
- `requires(condition)` is currently a function-level contract clause only
- `ensures(condition)` is currently a function-level contract clause only
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
- UFCS / method-call sugar:
  - `value.scale(10.0)`
  - `sensor.clamp(min = 0.0, max = 1.0)`
- pipeline chains:
  - `value |> stage()`
  - `value |> stage(arg)`
  - `value |> (x => expr)`
- where-clause suffix:
  - `sqrt(a + b) where a = x * x, b = y * y`
  - `total where total: f64 = 1.0`
- range literals:
  - `0..10`
  - `0..=10`
- short lambda immediate-call sugar:
  - `(x => expr)(arg)`
- tuple literals:
  - `(1, true)`
  - `(value, ready, 1.0)`
- record literals:
  - `DecisionContext { camera: T, quality: 0.75 }`
  - `DecisionContext { quality: 0.75, camera: T }`
- record field access:
  - `ctx.camera`
  - `ctx.quality`
- record copy-with:
  - `ctx with { quality: 1.0 }`
  - `ctx with { camera: F, quality: 0.25 }`
  - `ctx with { quality }`
- record punning shorthand:
  - `DecisionContext { camera, quality }`
  - `let DecisionContext { camera, quality: _ } = ctx;`
  - `let DecisionContext { camera: T, quality } = ctx else return;`
- tuple types:
  - `(i32, bool)`
  - `(f64, quad, bool)`
- block expressions with a trailing tail value:
  - `{ let x = 1; x }`
- `if` expressions with explicit `else` blocks:
  - `if ready { 1.0 } else { 0.0 }`
- `match` expressions with value-producing arms:
  - `match state { T => { 1.0 } _ => { 0.0 } }`
  - `match state { T if ready == true => { 1.0 } _ => { 0.0 } }`
- `loop` expressions with explicit `break value`:
  - `loop { break 1.0; }`
  - `loop { if ready { break 1.0; } else { break 0.0; } }`
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
- tuple literal arity must be at least 2 in the current contract

Current v0 range-literal limits:

- range literals currently accept only `i32` bounds
- `start..end` is half-open and `start..=end` is closed
- range literals currently lower through an internal executable interval
  descriptor rather than a dedicated runtime range opcode
- range equality is not yet part of the stable source contract
- range literals are not yet part of the stable tuple/user-data surface
- `for ... in range` currently exposes only the narrow `i32` interval surface
- descending/custom-step/general iterable range forms are not yet part of the
  stable syntax contract

Current v0 tuple limits:

- tuples are currently aggregate value carriers only
- tuple literals and tuple types are supported
- tuple destructuring bind is currently statement-level only
- tuple destructuring bind currently supports only flat name-or-`_` item lists
- tuple `let-else` currently supports only flat name/`_`/quad-literal item
  lists
- tuple destructuring bind currently requires arity at least 2
- tuple destructuring assignment is currently statement-level only
- tuple destructuring assignment currently supports only flat name-or-`_` item lists
- tuple equality follows ordinary `==` / `!=` when both operands have the same
  tuple type
- tuple field access and tuple pattern matching beyond flat destructuring bind
  are not yet part of the stable surface

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
- required non-default parameters must still be supplied exactly once in v0
- named arguments reorder to the declared parameter order before ordinary
  type-checking and lowering
- named arguments are not yet part of the builtin-call surface

Current first-wave `requires` rules:

- `requires(condition)` currently attaches only to ordinary user-defined
  function declarations
- `requires(condition)` may appear on block-bodied and expression-bodied
  functions
- each `condition` must be `bool`
- the current stable subset allows only parameter references, tuple literals,
  record field reads, and pure unary/binary operator expressions
- call expressions, block/control-flow expressions, range literals, record
  construction, and record copy-with are not part of this slice

Current first-wave `ensures` rules:

- `ensures(condition)` currently attaches only to ordinary user-defined
  function declarations
- `ensures(condition)` may appear on block-bodied and expression-bodied
  functions
- each `condition` must be `bool`
- the current stable subset allows parameter references, optional synthetic
  `result` binding, tuple literals, record field reads, and pure unary/binary
  operator expressions
- the synthetic `result` binding is reserved while `ensures` clauses are
  present and is available only for non-unit returns
- call expressions, block/control-flow expressions, range literals, record
  construction, and record copy-with are not part of this slice

Current UFCS / method-call rules:

- `receiver.name(args...)` is accepted as postfix call sugar
- UFCS currently desugars to ordinary call order: `name(receiver, args...)`
- UFCS may chain because it remains ordinary expression/call surface after
  desugaring
- `.name` without `(...)` is parsed as field access and then typechecked only
  against nominal record values
- UFCS does not introduce object members or method declarations

Current where-clause rules:

- `expr where name = value, ...` is accepted as expression-suffix sugar
- each `where` binding currently follows ordinary `let` spelling with optional
  whole-binding type annotation
- bindings appear in source order and are visible to later `where` bindings and
  to the tail expression
- `where` currently desugars through the existing block-expression path

Current loop-expression rules:

- `loop { ... }` is accepted only as an expression form in this slice
- loop-expression bodies currently exit only through `break expr;`
- bare `break;` is not part of the stable v0 contract
- `break expr;` is currently valid only inside `loop` expression bodies

Current honest limit:

- `where` is currently expression-suffix sugar only
- `where` bindings currently use ordinary local names only; tuple/record
  destructuring is not yet part of the stable `where` contract
- `loop` is currently expression-only; statement-loop, `continue`, and bare
  `break;` are not yet part of the stable contract

Current default-parameter rules:

- only trailing parameters may declare defaults in v0
- omitted arguments may currently be filled only from those declared trailing
  defaults
- default initializers are part of ordinary user-defined function declarations
- builtin calls do not expose default-parameter surface

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
- user-defined aggregate value operations beyond top-level nominal record declarations
- collections as first-class language forms
- generics or trait-like abstraction
- exceptions or Python-style dynamic execution
- concurrency-oriented source constructs

## Contract Rule

Any public change to source syntax, source statement forms, or operator meaning
should update this document in the same change series.
