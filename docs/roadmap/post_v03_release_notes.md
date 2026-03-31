# Semantic Post-v0.3 Release Notes

Status: prerelease checkpoint

These notes summarize the current release-ready surface after `v0.1`, `v0.2`,
and `v0.3` have all landed in `main`. They do not themselves cut a version or
publish assets.

## Exact Source Commit

- `main` freeze checkpoint: `4fec82f`

## Ready Surfaces

### Language Surface

- expression-valued control:
  - block expressions
  - `if` as expression
  - `match` as expression
- guarded control:
  - match guards
  - guard clause
- composition and call density:
  - pipeline operator
  - expression-bodied functions
  - capture-free short lambdas
  - named arguments
  - default parameters
- flow/data density:
  - range literals
  - `for ... in range`
  - tuple literals and tuple types
  - tuple destructuring bind/assignment
  - tuple `let-else`
- contract visibility and compile-time density:
  - `assert`
  - `const`
  - extended numeric literals

### Contract And Data Core

- first-wave function contracts:
  - `requires(condition)`
  - `ensures(condition)`
  - narrow `invariant(condition)`
- nominal ADT declarations and constructors
- ADT match core and exhaustiveness enforcement
- `Option(T)` and `Result(T, E)` standard forms
- `Option::Some/None` and `Result::Ok/Err` match ergonomics
- first-wave units of measure over supported numeric families

### Record Layer

- canonical nominal record declarations
- record literals
- read-only field access
- pass/return and equality-safe comparisons
- explicit record destructuring bind
- narrow record `let-else`
- immutable copy-with update
- shorthand/punning ergonomics on canonical record forms

### Schema And Boundary Core

- canonical `schema` declarations:
  - record-shaped
  - tagged-union
  - role-marked `config schema`, `api schema`, `wire schema`
  - optional `version(<u32>)`
- deterministic validation-plan ownership and generated validation checks
- canonical config-document parsing and schema validation in `smc-cli`
- deterministic generated API-contract artifacts
- deterministic schema compatibility classification and migration metadata
- deterministic generated wire-contract artifacts:
  - tagged wire unions
  - record patch types

## Current Known Limits

- no new release tag has been cut from this checkpoint yet
- published asset validation for the next tag has not yet been recorded
- generated API and wire contracts are review/build artifacts, not runtime
  transport engines
- wire patch types are review metadata only; there is no runtime patch
  application engine
- no widening of `prom-*`, host capability, or runtime boundaries is implied by
  these waves
- the `smlsp` / workbench protocol bridge remains outside this release note
  unless promoted separately

## Required Before Tag Cut

- run `cargo test --workspace` on the exact release candidate commit
- run `cargo test --test public_api_contracts`
- run `pwsh -File scripts/verify_release_bundle.ps1`
- validate downloaded release assets against
  `docs/roadmap/release_asset_smoke_matrix.md`
- decide the next forward version/tag without rewriting existing stable history

## Notes

- canonical language/source contract remains centered in:
  - `docs/spec/syntax.md`
  - `docs/spec/types.md`
  - `docs/spec/source_semantics.md`
  - `docs/spec/diagnostics.md`
  - `docs/spec/modules.md`
  - `docs/spec/logos.md`
- post-`v0.3` freeze governance note lives in:
  - `docs/roadmap/language_maturity/release_freeze_post_v03_checkpoint.md`
