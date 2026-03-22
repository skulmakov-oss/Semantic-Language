# Source Diagnostics Specification

Status: draft v0
Primary frontend owners: `sm-front`, `sm-sema`

## Purpose

This document defines the current public diagnostics contract for Semantic
source programs.

It covers source-facing parse, policy, type, and module/linkage diagnostics. It
does not redefine verifier, VM, or host-ABI error reporting.

## Diagnostic Layers

The current source toolchain exposes diagnostics from distinct layers:

1. Rust-like and Logos parsing in `sm-front`
2. source-level type checking in `sm-front`
3. module/import/export and Logos semantic checks in `sm-sema`

These layers are all source-facing, but they do not yet share one fully frozen
diagnostic code taxonomy.

## Frontend Parse Diagnostics

The parser already emits rendered source diagnostics with code-bearing messages
for many syntax and Logos-surface failures.

Current examples include:

- `E0200` aggregated Logos parse failures
- `E0201` and nearby codes for `System` parsing
- `E0210` to `E0216` for `Entity` parsing
- `E0220` to `E0233` for `Law` and `When` parsing
- `E0234` to `E0237` for type/expression-level parser expectations
- block-expression parse failures such as missing trailing tail values
- `if`-expression parse failures such as missing `else` or rejected `else if`
  sugar in value position
- expression-bodied function parse failures such as missing trailing `;`
- pipeline parse failures such as missing function-stage targets after `|>`
- short-lambda parse failures such as standalone non-invoked lambdas
- short-lambda surface failures such as rejected outer-local capture in v0
- `guard`-clause parse failures such as missing `else return`
- `match`-expression parse failures such as invalid literal arm patterns

Current guarantees:

- parser diagnostics include source position
- many parser diagnostics include rendered line/column context
- keyword mistakes may include case-insensitive suggestions

Current honest limit:

- the repository does not yet claim that every parser diagnostic code or exact
  wording is frozen as a long-term compatibility promise

## Policy Diagnostics

The frontend distinguishes ordinary syntax failures from policy rejections.

Current rules:

- policy-gated rejections are surfaced as `policy violation: ...`
- `FrontendErrorKind` currently distinguishes `Syntax` from
  `PolicyViolation`
- policy rejections cover source features disabled by the active parser profile

Current examples include:

- `f64` surface disabled by profile policy
- Logos surface disabled by profile policy
- legacy compatibility paths disabled by profile policy

## Type Diagnostics

The Rust-like type checker currently reports source-level type failures as plain
messages rather than as a stable numeric code family.

Current message families include:

- unknown variable
- unknown assignment target
- assignment to const binding
- unknown function
- argument count mismatch
- argument type mismatch
- invalid `assert` argument count
- invalid `assert` condition type
- statement-only `assert` used in value position
- let-binding type mismatch
- discard-binding type mismatch
- non-const-safe initializer in const declaration
- return type mismatch
- invalid `guard` condition type
- invalid `if` condition type
- `if`-expression branch type mismatch
- invalid `match` guard condition type
- `match`-expression branch type mismatch
- invalid `match` scrutinee or missing `_` arm
- unsupported statement forms inside a value-producing block
- unsupported operator for a type family
- explicit `fx` gap messages for still-narrow source cases

Current honest limit:

- exact wording of type-check messages is not yet a fully frozen compatibility
  contract
- users should treat the failure class as stable before treating the full text
  as stable

## Module And Linkage Diagnostics

The current module/import/export surface carries the most explicit stable source
diagnostic codes in the language contract.

Current public codes include:

- `E0238` cyclic imports
- `E0239` import read/parse/load failures
- `E0240` re-export policy violation
- `E0241` alias or binding collisions
- `E0242` public re-export collisions
- `E0243` symbol re-export cycles
- `E0244` missing selected import symbol
- `E0245` duplicate select alias, wildcard/select conflict, or kind mismatch

Current guarantees:

- these diagnostics are rendered as source-level semantic errors
- line/column information is preserved where available
- repository tests exercise these failure families directly

## Logos Semantic Diagnostics

The Logos path also emits semantic warnings and errors.

Current warning families include:

- `W0250` non-idiomatic law naming
- `W0251` large law
- `W0252` unused entity field
- `W0253` magic-number style warning

Current rule:

- warnings are part of the source contract, but they do not currently block
  execution in the same way as source errors

## Diagnostic Stability Rules

Current stability expectations:

- source diagnostics should remain deterministic for the same input and active
  profile
- introducing a new stable diagnostic code family should update this document
- changing the meaning of an existing module/import/export code should update
  this document in the same change series

Current honest limits:

- not every frontend type error has a stable numeric code yet
- exact prose wording is not fully frozen across all source layers

## Non-Goals

This document does not yet claim stable support for:

- localized diagnostics
- machine-readable JSON diagnostics as a frozen public API
- a single complete numeric-code taxonomy for every frontend/type error

## Contract Rule

Any public change to source-facing diagnostic families, stable source error
codes, or policy/type/module diagnostic boundaries should update this document
in the same change series.
