# Semantic Post-v0.3 Release Notes

Status: published stable release notes

These notes summarize the published stable release surface after `v0.1`,
`v0.2`, and `v0.3` all landed in `main`.

## Exact Source Commit

- published stable release commit: `087f2f6`

## Published Stable Tag

- `v1.1.1`

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

- generated API and wire contracts are review/build artifacts, not runtime
  transport engines
- wire patch types are review metadata only; there is no runtime patch
  application engine
- no widening of `prom-*`, host capability, or runtime boundaries is implied by
  these waves
- the `smlsp` / workbench protocol bridge remains outside this release note
  unless promoted separately

## Validation Contour Used For Release

- `cargo test --workspace`
- `cargo test --test public_api_contracts`
- `cargo test --test golden_semcode`
- `cargo test --test prometheus_runtime_matrix`
- `cargo test --test prometheus_runtime_goldens`
- `cargo test --test prometheus_runtime_negative_goldens`
- `cargo test --test prometheus_runtime_compat_matrix`
- `pwsh -File scripts/verify_release_bundle.ps1 -ManifestPath artifacts/release/post_v03_release_bundle_manifest.json`
- downloaded-asset smoke matrix rerun for exact tag `v1.1.1`

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
- version-cut decision note lives in:
  - `docs/roadmap/language_maturity/release_version_cut_decision.md`
- published release URL:
  - [Semantic v1.1.1](https://github.com/skulmakov-oss/Semantic-Language/releases/tag/v1.1.1)
