# Semantic v1 Milestones

- `M0 Repository Discipline`
  - architecture docs
  - crate map
  - legacy freeze policy
  - PR/review governance
  - PR template
  - architecture review checklist
  - current status: baseline largely in place
- `M1 Core Contract`
  - `sm-verify`
  - `SymbolId` runtime
  - quotas and admit-then-execute
  - current status: strong baseline exists in code and tests
- `M2 Language Completion`
  - `u32` / `fx`
  - optimizer minimum set
  - `sm-profile`
  - type completeness matrix
  - optimizer review checklist
  - current stable-note checkpoint: `docs/roadmap/language_maturity/fx_numeric_contract_notes.md`
  - current completed post-stable expansion checkpoint:
    `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`
  - current active post-stable units checkpoint:
    `docs/roadmap/language_maturity/units_of_measure_scope.md`
- `M3 Platform Formalization`
  - spec bundle
  - stable CLI
  - version contracts
  - current status: broad formalization baseline exists and owner alignment is already reflected in code
  - optimizer owner is fixed to `sm-ir` for the current `v1` baseline
  - SemCode format owner is fixed to `sm-ir` for the current `v1` baseline
  - public CLI owner is fixed to `smc-cli` for the current `v1` baseline
  - `docs/spec/syntax.md`
  - `docs/spec/types.md`
  - `docs/spec/ir.md`
  - `docs/spec/verifier.md`
  - `docs/spec/semcode.md`
  - `docs/spec/profile.md`
  - `docs/spec/vm.md`
  - `docs/spec/quotas.md`
  - `docs/spec/cli.md`
  - current completed post-stable source-contract freeze checkpoint:
    `docs/roadmap/language_maturity/source_language_contract.md`
  - current active IR hardening checkpoint:
    `docs/roadmap/language_maturity/ir_v1_contract_freeze.md`
  - current active SemCode version-discipline checkpoint:
    `docs/roadmap/language_maturity/semcode_version_discipline.md`
- `M4 PROMETHEUS Boundary`
  - ABI
  - capabilities
  - gates
  - current state: narrow working boundary exists and is fixed as the official `v1` scope
  - first-wave post-stable host-call expansion is now completed on `main`
  - published `v1.1.1` still keeps the narrower boundary as its official stable commitment
  - current completed post-stable expansion checkpoint:
    `docs/roadmap/language_maturity/prometheus_host_call_expansion_scope.md`
  - `docs/spec/abi.md`
  - `docs/spec/capabilities.md`
  - `docs/spec/gates.md`
- `M5 Semantic Runtime`
  - state
  - rules
  - orchestration
  - audit
  - semantic runtime integration checklist
  - `docs/spec/runtime.md`
  - `docs/spec/state.md`
  - `docs/spec/rules.md`
  - `docs/spec/audit.md`
  - current state: owner-split runtime baseline exists
  - richer runtime semantics remain non-blocking for the current narrow `v1`
  - current completed post-stable persistence/replay checkpoint:
    `docs/roadmap/language_maturity/persistence_replay_backend_scope.md`
  - current completed post-stable rule execution checkpoint:
    `docs/roadmap/language_maturity/rule_side_effect_execution_scope.md`
  - current completed post-stable replay expansion checkpoint:
    `docs/roadmap/language_maturity/multi_session_replay_archive_scope.md`
  - current completed post-stable rollback checkpoint:
    `docs/roadmap/language_maturity/rollback_persistence_semantics_scope.md`
  - current active runtime-boundary hardening checkpoint:
    `docs/roadmap/language_maturity/runtime_boundary_hardening.md`
- `M6 v1 Lockdown`
  - freezes
  - golden baselines
  - validation matrix
  - `tests/prometheus_runtime_matrix.rs`
  - `tests/prometheus_runtime_goldens.rs`
  - `tests/prometheus_runtime_negative_goldens.rs`
  - `tests/prometheus_runtime_compat_matrix.rs`
  - `docs/roadmap/runtime_validation_policy.md`
  - `docs/roadmap/v1_readiness.md`
  - `docs/roadmap/release_bundle_checklist.md`
  - `docs/roadmap/compatibility_statement.md`
  - current state: validation artifacts, CI-enforced release gates, and the
    published stable `v1.1.1` governance baseline all exist on `main`
  - current stable-note checkpoints:
    - `docs/roadmap/language_maturity/release_version_cut_decision.md`
    - `docs/roadmap/language_maturity/forward_stable_release_tag_policy.md`
- `M7 UI Application Boundary`
  - desktop window lifecycle
  - explicit UI capability/admission ownership
  - deterministic event polling and frame lifecycle
  - minimal draw-command family and one canonical demo application
  - current status: completed post-stable milestone
  - scope checkpoint:
    `docs/roadmap/language_maturity/ui_application_boundary_scope.md`
- `M8 Everyday Expressiveness Foundation`
  - text / strings
  - package ecosystem baseline
  - collections
  - first-class closures
  - current status: completed post-stable language-maturity package
  - planning docs:
    - `docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`
    - `docs/roadmap/language_maturity/m8_everyday_expressiveness_blueprint.md`
    - `docs/roadmap/language_maturity/m8_everyday_expressiveness_phased_implementation_plan.md`
  - current completed first subtrack:
    `docs/roadmap/language_maturity/text_type_full_scope.md`
  - current completed second subtrack:
    `docs/roadmap/language_maturity/package_ecosystem_baseline_scope.md`
  - current completed third subtrack:
    `docs/roadmap/language_maturity/collections_surface_full_scope.md`
  - current completed fourth subtrack:
    `docs/roadmap/language_maturity/first_class_closures_full_scope.md`
  - planning rule:
    - keep package baseline earlier than broad abstraction machinery
    - keep one active stream at a time
    - keep UI/platform expansion separate from language-maturity work
- `M9 General Abstraction Layer`
  - generics / parametric polymorphism
  - traits / protocols / interfaces
  - iterable abstraction
  - richer pattern surface
  - current status: completed post-stable language-maturity package
  - completed subtracks:
    - M9.1: `docs/roadmap/language_maturity/generics_full_scope.md`
    - M9.2: `docs/roadmap/language_maturity/traits_full_scope.md`
    - M9.3: `docs/roadmap/language_maturity/iterable_abstraction_full_scope.md`
    - M9.4: richer pattern surface (Wildcard, Or, IntRange, nested tuple, if-let)
  - planning rule:
    - keep one active stream at a time
    - keep UI/platform expansion separate from language-maturity work
