# Semantic v1 Backlog Seed

Current release-control wave:

- stabilize repository history for current `M3-M6` state
- keep new feature work paused until baseline and ownership decisions are fixed

Current decision wave:

- decide optimizer owner: `sm-opt` vs `sm-ir`
- decide SemCode format owner: `sm-emit` vs `sm-ir/local_format`
- decide CLI owner: `smc-cli` vs root `smc`
- decide narrow vs full `M4` scope for `v1`

Current blocking implementation wave:

- complete `fx` end-to-end
- align code layout with chosen owners
- add `M6` CI enforcement and release gates

Foundational work already in place:

- repository discipline and architecture baseline
- verifier and admit-then-execute baseline
- `SymbolId` runtime model and quota enforcement
- type completeness matrix and `u32` completion
- canonical `sm-profile`
- narrow PROMETHEUS boundary and owner-split semantic runtime baseline

Rule of execution:

- do not start semantic runtime before verifier, runtime purity, and quotas are in place;
- do not start ownership cleanup before the ownership decision issues land;
- one PR equals one logical step;
- contract/spec/tests come before cleanup and optimization.
