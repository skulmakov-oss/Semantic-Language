# Semantic v1 Compatibility Statement

Status: active stable release baseline

This document summarizes the current compatibility commitments for the
repository state published on the active Semantic stable line.

## SemCode Compatibility

Current compatible SemCode families:

- `SEMCODE0`
- `SEMCODE1`
- `SEMCODE2`
- `SEMCODE3`

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
- wider planned calls such as `StateQuery`, `StateUpdate`, `EventPost`, and `ClockRead` are explicitly outside the current `v1` compatibility envelope

## Explicit Non-Commitments

The repository does not yet claim final compatibility guarantees for:

- richer `fx` arithmetic semantics beyond the current stable value-transport and
  equality contract
- wider planned PROMETHEUS host-call families beyond `GateRead`, `GateWrite`, and `PulseEmit`
- persistence backends
- multi-session replay archives
- rule-side effect execution semantics beyond the current narrow orchestration contract
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
