# Semantic v1 Backlog Seed

Current release-control wave:

- keep the active stable release line stable on `main`
- keep new feature work paused while release-facing docs, asset smoke checks, and packaging stay aligned

Current release-maintenance wave:

- keep `blueprint`, `milestones`, `backlog`, and `v1_readiness` aligned with the published stable line
- keep published release assets validated against representative source programs
- keep release notes and compatibility statements honest about current narrow `v1` limits

Current remaining `v1` wave:

- tighten remaining `fx` numeric contract notes now that the canonical value path is end-to-end
- keep forward stable release/tag policy explicit without rewriting history

Current post-`v1` wave:

- richer `fx` arithmetic beyond the current value path
- wider PROMETHEUS host-call families beyond the narrow `v1` boundary
- persistence and replay backends
- richer rule-side effect execution semantics
- keep the explicit `ton618_core` / `ton618-core` compatibility perimeter narrow and documented

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
