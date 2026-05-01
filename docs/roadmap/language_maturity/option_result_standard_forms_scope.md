# Option and Result Standard Forms Scope

Status: completed first-wave baseline history

## Purpose

Freeze the decision boundary for `V02-06` as completed first-wave baseline
history for first-class `Option` and `Result`.

This document exists because `#117` is not just "more ADT work". The current
language now has:

- nominal ADT declarations
- ADT constructors
- ADT match core
- first-wave exhaustiveness checks

But it still does **not** have a general generic type system. Without an
explicit checkpoint, it is too easy to accidentally turn `#117` into:

- a hidden prelude/type injection story
- a stealth generics feature
- or a broad standard-library/runtime expansion

## Current Landed State

The current `main` already includes:

- canonical ADT declarations and constructors
- canonical `MakeAdt` lowering through verifier and VM
- ADT `match` core with explicit `Enum::Variant(...)` patterns
- first-wave exhaustive enum `match` enforcement

The current source type grammar still remains intentionally narrow:

- primitives such as `quad`, `bool`, `i32`, `u32`, `fx`, `f64`
- nominal record or enum names
- tuple types
- range carrier types

It does **not** include:

- angle-bracket generics
- hidden standard-form injection into the ADT table
- a general higher-kinded or type-parameterized declaration model

## Why A Separate Scope Checkpoint Is Needed

`Option` and `Result` are listed as first-class standard forms in `#117`, but
there are two materially different ways to interpret that requirement:

1. narrow standard forms that reuse the current ADT execution path
2. a wider generic/type-constructor language feature

Those are not the same amount of work or risk.

If the project wants to keep `V02-06` inside `v0.2`, the first wave must stay
inside the existing language and contract surface.

## Recommended First-Wave Shape

If `#117` remains in `v0.2`, keep it narrow and explicit.

Recommended first-wave model:

- add standard-form type syntax `Option(T)` and `Result(T, E)`
- keep type arguments inside ordinary parenthesized type grammar, not angle
  brackets
- keep constructor surface canonical and explicit:
  - `Option::Some(value)`
  - `Option::None`
  - `Result::Ok(value)`
  - `Result::Err(error)`
- treat those forms as built-in standard families in sema/typechecking rather
  than as user-declared generic enums
- lower them through the existing canonical ADT-style carrier path
- support `match` ergonomics only through the already-canonical explicit
  variant patterns

This keeps the first wave honest:

- type syntax becomes slightly richer
- the execution path stays canonical and inspectable
- no separate generic runtime or host boundary is introduced

## Explicit Non-Goals For First-Wave `#117`

Do not include any of the following in the first implementation wave:

- general generic type parameters
- angle-bracket type application
- user-defined parameterized enums or records
- hidden prelude declarations that silently materialize user-visible ADTs
- special host ABI widening for `Option` or `Result`
- shorthand match patterns beyond canonical explicit variant form
- call-boundary or exception semantics disguised as `Result` ergonomics

If any of those become necessary, they should be treated as a later expansion
issue, not as part of `V02-06`.

## Completed Decision

`#117` is complete on `main` as the narrow standard-forms wave:

1. explicit `Option(T)` / `Result(T, E)` typing is admitted,
2. canonical constructors and variant patterns are supported,
3. verified success/none/error execution coverage exists,
4. the implementation stays inside the existing ADT-style carrier path rather
   than widening into a general generic runtime.

## Acceptance Criteria For A Narrow Standard-Forms Wave

- parser accepts `Option(T)` and `Result(T, E)` in declared type positions
- sema validates those forms without introducing a general generic system
- constructor typing works for `Some` / `None` / `Ok` / `Err`
- `match` over those standard forms reuses the canonical ADT pattern path
- lowering stays on one inspectable carrier family
- docs and diagnostics describe the exact first-wave boundary
- verified-path tests cover success, none, and error flows

## Freeze Rule

The completed first wave remains narrow:

- `Option(T)` and `Result(T, E)` stay standard forms, not a reopening of
  general user-defined parameterized ADTs,
- constructor and pattern ergonomics remain canonical and explicit,
- any widening into broader generic, prelude-injection, or call-boundary
  semantics requires a new explicit track.
