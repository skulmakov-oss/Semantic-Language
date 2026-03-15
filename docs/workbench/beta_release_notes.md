# Semantic Workbench Beta Release Notes

Semantic Workbench is currently published as a beta-facing desktop shell over
the Semantic repository. These notes describe only the workflows that exist on
the current `main` line.

## Current Beta Posture

- package format: portable Windows zip
- desktop shell: Tauri + React
- command orchestration: `smc`, `svm`, `cargo`, and release scripts only
- repository truth: `docs/spec/*`, `docs/roadmap/*`, release artifacts, and
  captured validation outputs

These notes do not claim a second source of truth for compiler, verifier, VM,
runtime, or release semantics.

## Setup

### Development setup

```powershell
cd apps\workbench
npm install
npm run dev
```

### Local validation

```powershell
cd apps\workbench
npm run lint
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri:build -- --debug --no-bundle
```

### Portable beta packaging

```powershell
pwsh -File scripts/package_workbench_beta.ps1
```

Packaging evidence is recorded under `artifacts/workbench/beta-smoke/`.

## Stable Now

The following Workbench workflows are considered beta-ready on the current
mainline:

- open repository or canonical sub-workspace
- recent workspaces and local settings restore
- overview and operations cockpit
- jobs panel and command output
- spec navigator and release/readiness panels
- project explorer and editor shell
- current-file `check` and `compile`
- diagnostics grouping and source navigation
- formatter integration through canonical `smc fmt`
- disasm / verify / trace / runtime inspectors
- release console and clean validation report export
- project scaffolding through canonical `Semantic.toml` / `src/main.sm`
- docs entry and package metadata preview

## Experimental

The following path remains explicitly experimental:

- `smlsp` protocol bridge

Experimental means:

- it stays behind the Workbench experimental toggle
- it is not required for the primary beta authoring loop
- failures in this path must remain local to the experimental panel
- it must not be described as stable editor semantics

## Known Limits

Current beta known limits:

- packaging is a portable zip, not a full installer
- package smoke evidence is currently Windows-first
- `smlsp` is optional and experimental
- Workbench is not a full IDE and does not claim complete editor-protocol
  coverage
- Workbench does not own compiler, verifier, VM, PROMETHEUS, or runtime logic
- release status is derived from canonical docs and recorded validation outputs,
  not from a hidden UI-only score

## What These Notes Do Not Promise

These notes do not promise:

- a full stable installer pipeline
- a public guarantee for `smlsp`
- richer editor semantics beyond the current bridge
- any capability beyond the current `main` branch implementation
- ownership transfer of Semantic internals into Workbench

## Evidence Pointers

- packaging contract: `docs/workbench/beta_packaging.md`
- latest beta smoke report:
  `artifacts/workbench/beta-smoke/workbench_beta_smoke_latest.md`
- latest beta package manifest:
  `artifacts/workbench/beta-smoke/workbench_beta_package_manifest.json`
