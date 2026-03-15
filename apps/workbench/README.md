# Semantic Workbench

Semantic Workbench is the desktop orchestration shell for the Semantic
repository.

This app is intentionally scoped as a UI and command-orchestration layer over
public Semantic surfaces. It does not own compiler, verifier, VM, PROMETHEUS,
or release semantics.

## Current Slice

This bootstrap slice provides:

- React + TypeScript frontend shell
- Tauri desktop wrapper
- route layout for overview, project, spec, diagnostics, inspect, release, and
  settings
- configuration for local dev and debug builds

## Commands

```powershell
npm install
npm run dev
npm run lint
npm run build
npm run tauri:build -- --debug --no-bundle
```

## Beta Packaging

```powershell
pwsh -File ..\..\scripts\package_workbench_beta.ps1
```

This builds the release executable, creates a portable beta zip, launches the
packaged app for a short smoke window, and records evidence under
`artifacts/workbench/beta-smoke/`.

## Scope Guard

The first implementation waves must continue to respect the repository rule that
Workbench talks to Semantic through:

- `smc`
- `svm`
- `cargo`
- public release scripts

Direct private crate coupling is out of scope.
