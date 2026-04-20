# Semantic v1 Backlog Seed

Current release-control wave:

- keep the active stable release line stable on `main`
- keep new feature work paused while release-facing docs, asset smoke checks, and packaging stay aligned
- keep active engineering work anchored to the canonical `main` source of truth
  in
  `docs/roadmap/language_maturity/mainline_source_of_truth_policy.md`

Current release-maintenance wave:

- keep `blueprint`, `milestones`, `backlog`, and `v1_readiness` aligned with the published stable line
- keep published release assets validated against representative source programs
- keep release notes and compatibility statements honest about current narrow `v1` limits

Current remaining `v1` wave:

- `fx` numeric contract notes are frozen for the current stable line in
  `docs/roadmap/language_maturity/fx_numeric_contract_notes.md`
- forward stable release/tag policy is frozen for the current stable line in
  `docs/roadmap/language_maturity/forward_stable_release_tag_policy.md`

Current post-`v1` wave:

- `Runtime Ownership (tuple + direct record-field paths)` is completed and now
  lives as frozen baseline history in `docs/spec/runtime_ownership.md`
- `M7 UI Application Boundary` is now completed as first-wave baseline history
  and is scoped in
  `docs/roadmap/language_maturity/ui_application_boundary_scope.md`
- the language-maturity package after the completed post-stable runtime waves
  is documented in:
  - `docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`
  - `docs/roadmap/language_maturity/m8_everyday_expressiveness_blueprint.md`
  - `docs/roadmap/language_maturity/m8_everyday_expressiveness_phased_implementation_plan.md`
- `M8.1 Text / String Surface` is completed as first-wave baseline history in
  `docs/roadmap/language_maturity/text_type_full_scope.md`
- `M8.2 Package Ecosystem Baseline` is now completed as first-wave baseline
  history and is scoped in
  `docs/roadmap/language_maturity/package_ecosystem_baseline_scope.md`
- `M8.3 Collections Surface` is now completed as first-wave baseline history
  and is scoped in
  `docs/roadmap/language_maturity/collections_surface_full_scope.md`
- `M8.4 First-Class Closures` is now completed as first-wave baseline history
  and is scoped in
  `docs/roadmap/language_maturity/first_class_closures_full_scope.md`
- `M9.1 Generics` is now completed as first-wave baseline history and is scoped
  in `docs/roadmap/language_maturity/generics_full_scope.md`
- `M9.3 Iterable Abstraction` is now completed as first-wave baseline history
  and is scoped in
  `docs/roadmap/language_maturity/iterable_abstraction_full_scope.md`
- `Source Language Contract Freeze` is completed and now lives as frozen
  baseline history in
  `docs/roadmap/language_maturity/source_language_contract.md`
- `NEXT-1..NEXT-4` post-base closure tracks are completed and now live as
  frozen baseline history in `docs/roadmap_next.md`
- the retained non-owning TON618 compatibility perimeter is completed and now
  lives as frozen baseline history in
  `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`
- the first-wave PROMETHEUS host-call expansion track is completed and now
  lives as frozen baseline history in
  `docs/roadmap/language_maturity/prometheus_host_call_expansion_scope.md`
- the first-wave persistence/replay backend track is completed and now lives as
  frozen baseline history in
  `docs/roadmap/language_maturity/persistence_replay_backend_scope.md`
- the first-wave rule-side effect execution track is completed and now lives as
  frozen baseline history in
  `docs/roadmap/language_maturity/rule_side_effect_execution_scope.md`
- the first-wave multi-session replay archive track is completed and now lives
  as frozen baseline history in
  `docs/roadmap/language_maturity/multi_session_replay_archive_scope.md`
- the first-wave rollback persistence semantics track is completed and now
  lives as frozen baseline history in
  `docs/roadmap/language_maturity/rollback_persistence_semantics_scope.md`
- the first-wave `fx` arithmetic expansion track is completed and now lives as
  frozen baseline history in
  `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`
- the first-wave `Option` / `Result` standard-forms track is completed and now
  lives as frozen baseline history in
  `docs/roadmap/language_maturity/option_result_standard_forms_scope.md`
- the IR/runtime anti-drift package is completed and now lives as frozen
  baseline history in:
  - `docs/roadmap/language_maturity/ir_v1_contract_freeze.md`
  - `docs/roadmap/language_maturity/semcode_version_discipline.md`
  - `docs/roadmap/language_maturity/runtime_boundary_hardening.md`

Current next-focus wave:

- `Executable Module Entry Scope` is now the active blocker-removal stream in
  `docs/roadmap/language_maturity/executable_module_entry_scope.md`
- this stream exists because `Gate 1` ended in `limited release` rather than a
  broader practical-programming claim
- do not widen the frozen source-language bundle beyond this narrow executable
  module entry track without a new track or an explicit source-contract
  amendment

Current qualification wave:

- `Gate 1 Release Qualification Protocol` in
  `docs/roadmap/release_qualification/gate1_protocol.md`
- the first `Gate 1` cycle is now completed through:
  - `reports/g1_real_program_trial.md`
  - `reports/g1_frontend_trust.md`
  - `reports/g1_execution_integrity.md`
  - `reports/g1_benchmark_baseline.md`
  - `reports/g1_surface_expressiveness.md`
  - `reports/g1_release_scope_statement.md`
- the current Gate 1 decision state is `limited release` for the admitted
  narrow practical-programming contour
- keep UI out of the first qualification contour unless UI is explicitly
  admitted into a future release scope
- do not treat landed-on-`main` behavior as automatically release-promised
- rerun or amend Gate 1 only through a new explicit qualification cycle if the
  admitted release contour is widened

Foundational work already in place:

- repository discipline and architecture baseline
- verifier and admit-then-execute baseline
- `SymbolId` runtime model and quota enforcement
- type completeness matrix and `u32` completion
- `fx` end-to-end value path and verified-path `f64` builtin coverage
- canonical `sm-profile`
- narrow PROMETHEUS boundary and owner-split semantic runtime baseline
- CI-enforced release bundle and compatibility checks

Rule of execution:

- do not start semantic runtime before verifier, runtime purity, and quotas are in place;
- do not reopen scope while the active stable line is being maintained;
- one PR equals one logical step;
- contract/spec/tests come before cleanup and optimization.
