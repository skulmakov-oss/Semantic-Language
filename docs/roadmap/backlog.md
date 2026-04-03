# Semantic v1 Backlog Seed

Current release-control wave:

- keep the active stable release line stable on `main`
- keep new feature work paused while release-facing docs, asset smoke checks, and packaging stay aligned

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

- richer `fx` arithmetic beyond the current value path:
  `docs/roadmap/language_maturity/fx_arithmetic_full_scope.md`

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
