# Semantic v1 Backlog Seed

Critical first wave:

- repository discipline and architecture baseline
- `sm-verify` contract and crate
- structural verification
- control-flow verification
- admit-then-execute bridge

Second wave:

- `SymbolId` runtime model
- `sm-runtime-core`
- VM symbol migration
- quota taxonomy and enforcement

Third wave:

- type completeness matrix
- `u32` / `fx` end-to-end
- optimizer minimum set
- `sm-profile` extraction and pipeline integration

Rule of execution:

- do not start semantic runtime before verifier, runtime purity, and quotas are in place;
- one PR equals one logical step;
- contract/spec/tests come before cleanup and optimization.
