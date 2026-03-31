# Semantic Release Asset Smoke Matrix

Status: prerelease asset validation baseline

This document records the minimum smoke validation required against downloaded release assets, not just locally built binaries.

## Current Validated Tag

- `v1.1.1-beta4`

## Next Required Rerun

- downloaded assets for candidate stable tag `v1.1.1`

## Validated Assets

- `smc.exe`
- `svm.exe`
- `semantic-language-windows-x64-v1.1.1-beta4.zip`

## Current Smoke Matrix

| Scenario | Source | Validation | Expected Signal | Current Result |
| --- | --- | --- | --- | --- |
| Minimal compile-run-disasm | generated `smoke_minimal.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE0`, `RET`, clean run | pass |
| Verified-path `f64` builtin pipeline | generated `smoke_builtin_f64.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE1`, `SUB_F64`, builtin `CALL`, clean run | pass |
| Heavy semantic policy trace | `examples/semantic_policy_overdrive_trace.sm` | `smc.exe compile`, `svm.exe run`, `svm.exe disasm` | `SEMCODE1`, `fusion_consensus_state`, `policy_trace_guard`, `policy_trace_quality`, `policy_trace` | pass |

## Current Validation Notes

- the minimal source compiled to `22 bytes`
- the builtin `f64` source compiled to `257 bytes`
- the semantic trace source compiled to `6029 bytes`
- downloaded `beta4` binaries were revalidated from downloaded assets, not only from the local build tree

## Smoke Commands

Representative command pattern:

- `smc.exe compile <source>.sm -o <source>.smc`
- `svm.exe run <source>.smc`
- `svm.exe disasm <source>.smc`

## Release Rule

Every published beta or final release should repeat this smoke matrix against
the downloaded assets for that exact tag.

The next required repetition for a stable cut is the downloaded asset set for
`v1.1.1`, not only the older `v1.1.1-beta4` bundle.

If a release asset fails this smoke matrix, the tag should be treated as packaging-invalid even if `cargo test --workspace` remains green in the repository.
