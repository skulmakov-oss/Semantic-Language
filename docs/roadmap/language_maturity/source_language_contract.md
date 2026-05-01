# Source Language Contract Freeze

Status: completed checkpoint

## Goal

Turn the current Semantic source surface into a single canonical language contract rather than a set of scattered implementation notes.

## Why

Semantic already has a strong execution contract. What it still lacks is a fully frozen source-language contract that a user can treat as the primary truth for writing programs.

## Canonical Source Contract Bundle

The current `main` source contract is frozen through the canonical spec bundle
and its adjacent source-facing companion:

- `docs/spec/syntax.md`
- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/diagnostics.md`
- `docs/spec/modules.md`
- `docs/LANGUAGE.md`

That bundle is the public truth for the current source surface.

## Included In This Freeze

- define canonical source syntax and grammar boundaries
- define expression and statement semantics
- define type rules for `quad`, `bool`, `i32`, `u32`, `f64`, and `fx`
- define import and export behavior as part of the language contract
- define user-facing diagnostics expectations for source-level contract violations
- freeze the current admitted post-stable source surface as documented in the
  canonical bundle, including:
  - records
  - schemas
  - generics
  - traits
  - iterable loops for built-in sequences and direct record `Iterable` impls

## Explicit Non-Goals

- adding new runtime semantics
- widening the PROMETHEUS boundary
- changing bytecode or VM behavior beyond what is required to document the source contract honestly
- claiming support for source features outside the canonical bundle
- reopening iterable scope beyond the completed first-wave contract
- widening source ownership, ADT payload, schema migration, or unit semantics
  beyond what the current spec bundle already states

## Freeze Rules

- `docs/spec/*` remains the source-language contract authority
- source-surface changes must update the relevant source-contract file in the
  same PR
- diagnostics and examples must stay aligned with the documented source surface
- source docs must not imply runtime, ABI, or VM behavior beyond what the
  execution-contract specs already publish
- unsupported source behavior must stay explicit rather than being left to
  implementation accident

## Done Boundary

This checkpoint is complete because:

- one canonical source-language specification bundle exists and is linked from the main spec index
- syntax, typing, and module semantics are documented in one intentional contract family
- examples and diagnostics match the documented source surface
- no major source feature remains described only by implementation behavior

Related staged design-target notes:

- `docs/roadmap/language_maturity/function_contract_invariant_scope.md`
- `docs/roadmap/language_maturity/config_schema_contract_scope.md`
- `docs/roadmap/language_maturity/generated_api_contract_surface_scope.md`
- `docs/roadmap/language_maturity/option_result_standard_forms_scope.md`
- `docs/roadmap/language_maturity/record_data_model.md`
- `docs/roadmap/language_maturity/record_scenarios.md`
- `docs/roadmap/language_maturity/range_execution_story.md`
- `docs/roadmap/language_maturity/schema_first_declarations_scope.md`
- `docs/roadmap/language_maturity/schema_versioning_and_migration_scope.md`
- `docs/roadmap/language_maturity/tagged_wire_unions_and_patch_types_scope.md`
- `docs/roadmap/language_maturity/units_of_measure_scope.md`
- `docs/roadmap/language_maturity/validation_derived_from_schemas_scope.md`
