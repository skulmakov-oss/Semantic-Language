# Workbench Beta Packaging

Workbench beta packaging is intentionally a thin release layer over the
integrated desktop shell. It does not introduce a second installer-specific
workflow or alternate runtime model.

## Current Beta Package

The current beta-facing package is a portable zip built from the Tauri release
executable:

- release build: `npm run tauri:build -- --no-bundle`
- portable package: zip of the produced `semantic-workbench-app.exe`
- smoke evidence: generated under `artifacts/workbench/beta-smoke/`

This keeps the packaging path reproducible on Windows machines that do not have
installer toolchains such as NSIS or WiX.

## Evidence Command

Run:

```powershell
pwsh -File scripts/package_workbench_beta.ps1
```

The script captures three evidence layers:

1. portable package inventory with file hashes
2. launch smoke for the packaged Workbench executable
3. representative command-loop smoke for diagnostics, format, compile, verify,
   disasm, run, and release bundle verification

## Evidence Outputs

The command writes:

- `artifacts/workbench/beta-smoke/workbench_beta_package_manifest.json`
- `artifacts/workbench/beta-smoke/workbench_beta_smoke_latest.json`
- `artifacts/workbench/beta-smoke/workbench_beta_smoke_latest.md`

It also preserves full stdout and stderr captures under
`artifacts/workbench/beta-smoke/logs/`.

## Scope Boundary

This beta packaging path does not claim a second source of truth for Workbench
readiness.

- package launch evidence comes from the packaged executable
- command-loop evidence comes from real `smc`, `svm`, and release-script runs
- canonical release truth remains in repository docs, release artifacts, and
  recorded validation outputs
