# Semantic v1 WBS Summary

Program:

- `1.0 Semantic v1`

Milestones:

- `1.1` repository discipline
- `1.2` core contract
- `1.3` language completion
- `1.4` platform formalization
- `1.5` PROMETHEUS boundary
- `1.6` semantic runtime
- `1.7` v1 lockdown
- `1.8` UI application boundary (post-stable)

Current post-stable focus:

- keep the published `v1.1.1` boundary honest while current `main` moves forward
- treat post-stable widening as explicit tracked streams rather than silent drift
- keep roadmap/spec/release-facing docs aligned with actual owner layers on `main`

Current non-blocking follow-up work:

- the active post-stable UI application boundary track is scoped in
  `docs/roadmap/language_maturity/ui_application_boundary_scope.md`
- its planned delivery waves are:
  - Wave 0: governance and owner split
  - Wave 1: boundary admission
  - Wave 2: desktop lifecycle
  - Wave 3: minimal drawing surface
  - Wave 4: freeze and close-out
- the retained non-owning TON618 compatibility perimeter is frozen as completed
  post-stable baseline history in
  `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`
- the first-wave PROMETHEUS host-call expansion is frozen as completed
  post-stable baseline history in
  `docs/roadmap/language_maturity/prometheus_host_call_expansion_scope.md`
- deepen runtime semantics only after `v1` scope is frozen
