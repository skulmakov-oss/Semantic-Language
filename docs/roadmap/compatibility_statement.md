# Semantic v1 Compatibility Statement

Status: active stable release baseline

This document summarizes the current compatibility commitments for the
repository state published on the active Semantic stable line.

## SemCode Compatibility

Current published-stable compatible SemCode families:

- `SEMCODE0`
- `SEMCODE1`
- `SEMCODE2`
- `SEMCODE3`

Current post-stable admitted families on `main`:

- `SEMCODE4`
- `SEMCODE5`
- `SEMCODE6`
- `SEMCODE7`

Current compatibility rule:

- standard execution accepts only verified SemCode
- verifier rejects unknown or unsupported SemCode headers
- VM must not silently reinterpret unsupported headers

## ParserProfile Compatibility

Current profile contract baseline:

- schema owner: `sm-profile`
- schema family: `ParserProfile`
- public version baseline: `1.0`

Current compatibility rule:

- semantic meaning changes require explicit profile contract review
- contract hash stability is required across canonical serialization roundtrips

## CLI Compatibility

Current compatibility-sensitive CLI surfaces:

- `smc profile show --json`
- `smc doctor --json`

Current compatibility rule:

- documented machine-readable fields must not change silently
- canonical CLI owner remains `smc-cli`; root process entrypoints must not become second CLI owners

## PROMETHEUS Runtime Compatibility

Current compatibility-sensitive `prom-*` surfaces:

- capability manifest schema/version
- gate registry validation behavior
- runtime session descriptor fields
- canonical audit event families used by orchestration helpers

Current compatibility rule:

- changes to these surfaces require:
  - spec updates
  - runtime matrix and golden updates
  - compatibility review
- boundary and public API CI guards must remain green for the current contract-sensitive crates

Current `v1` scope commitment:

- compatibility commitments for `prom-*` apply only to the narrow ABI/capability/gate surface already implemented in the repository
- post-stable admitted calls such as `StateQuery`, `StateUpdate`, `EventPost`,
  and `ClockRead` remain outside the current `v1.1.1` compatibility envelope
- support for those wider calls on current `main` is a forward-only repo-main
  contract, not a retroactive widening of the published stable tag
- the same forward-only reading also applies to persisted archive
  materialization/loading for `StateSnapshotArchive` and `AuditReplayArchive`
- the same forward-only reading also applies to first-wave multi-session replay
  archive ownership/materialization for `MultiSessionReplayArchive`
- the same forward-only reading also applies to first-wave rule-side effect
  execution for declared `StateWrite` and `AuditNote`
- the same forward-only reading also applies to first-wave rollback persistence
  semantics for `StateRollbackArtifact` ownership and deterministic
  `SemanticStateStore::apply_rollback(...)`

## Explicit Non-Commitments

The repository does not yet claim final compatibility guarantees for:

- richer `fx` arithmetic semantics beyond the current stable value-transport and
  equality contract
- any wider PROMETHEUS host-call families beyond the currently admitted
  first-wave post-stable pack
- replay archive semantics beyond the current admitted first-wave
  `MultiSessionReplayArchive` ownership/materialization contract
- rollback persistence semantics beyond the current admitted first-wave
  artifact ownership and deterministic apply/restore contract
- rule-side effect execution semantics beyond the current admitted first-wave
  declared `StateWrite` / `AuditNote` contract
- broader packaged release layout beyond the current stable assets

## Release Honesty Rule

This compatibility statement must stay aligned with:

- `docs/spec/`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/runtime_validation_policy.md`

If a surface is not yet fully stabilized, it must remain listed as a non-commitment rather than being implied as release-stable.

Published stable releases should keep this statement aligned with:

- the current tag notes
- packaged Windows assets (`smc.exe`, `svm.exe`, and bundled zip)
- the active `main` branch behavior
