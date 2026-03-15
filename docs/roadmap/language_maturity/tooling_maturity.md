# Tooling Maturity

Status: proposed v0

## Goal

Define the tooling bar required for Semantic to feel like a usable language
platform rather than only a compiler-plus-VM stack.

This workstream is about user-facing development experience around the language,
not about widening the core language only to justify tooling work.

## Current Baseline

Today Semantic has a real toolchain, but a narrow tooling story.

Current strengths:

- canonical CLI entrypoints through `smc` and `svm`
- compile, run, and disassembly workflows
- bytecode and runtime validation
- release smoke matrix and golden coverage

Current honest gaps:

- no canonical formatter contract
- no LSP/editor protocol story
- no first-class debugger workflow beyond disasm/verified-path inspection
- no source-doc generation contract
- no stable project scaffolding command

## Why This Matters

Rust and Python feel mature not only because they compile or run, but because
their tooling is predictable, discoverable, and integrated into everyday work.

Without a tooling roadmap, Semantic remains approachable only to users who are
already comfortable with its repository and internal workflows.

## Tooling Principles

The tooling layer should preserve the same discipline as the runtime layer.

Required principles:

- deterministic behavior
- explicit stability labels
- clear ownership boundaries between CLI and higher tooling
- no pretending that internal scripts are already public tooling contracts
- documentation for the workflows that users are actually expected to run

## Tooling Layers

The detailed tooling map is defined in:

- `docs/roadmap/language_maturity/tooling_layers.md`
- `docs/roadmap/language_maturity/tooling_workflows.md`

## Current Layering Rule

The first tooling maturity wave should separate these responsibilities clearly:

- CLI command surface
- source formatter
- editor/LSP integration
- debugger and trace tooling
- docs generation
- project scaffolding
- testing and profiling helpers

## Staged Plan

### Stage 1: Core Authoring Loop

The first useful maturity wave should establish:

- canonical formatter direction
- canonical editor/LSP direction
- clear compile/check/run/disasm workflows
- stable status labels for current tooling surfaces

### Stage 2: Debug And Inspection

After the authoring loop, the next wave should define:

- trace-oriented debugging workflows
- source-to-bytecode inspection workflows
- debug-symbol-aware tooling expectations

### Stage 3: Publishing And Project UX

After authoring and debugging, the next wave should define:

- docs generation
- package-aware scaffolding
- stable project bootstrap flows

## Non-Goals

This workstream is not intended to:

- promise every tool in one release cycle
- treat internal scripts as if they were already public tooling contracts
- widen the core language only to justify tooling work
- promise a full IDE ecosystem before the language/platform contracts are ready

## Acceptance Criteria

This workstream should be considered materially started only when:

- one canonical tooling roadmap exists
- CLI, formatter, language server, debugger, docs, and scaffolding
  responsibilities are separated clearly
- at least one editor-facing integration path is documented
- user-facing tooling commands are described as stable, draft, or experimental
  rather than left implicit

## Immediate Next Slice

The immediate next slice for this PR is to define:

- the tooling responsibility map
- the status matrix for current and proposed tooling
- the canonical workflows users should follow

because those pieces turn "tooling maturity" into an actual platform plan.
