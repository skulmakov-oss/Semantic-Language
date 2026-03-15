# Tooling Layers

Status: proposed v0

## Purpose

This document defines the intended responsibility map for the Semantic tooling
stack.

It separates current tooling layers by role so that formatter, CLI, LSP,
debugger, docs generation, and scaffolding do not collapse into one vague
"tooling" bucket.

## Layer 1: CLI Surface

Primary tools:

- `smc`
- `svm`

Purpose:

- canonical command-line entrypoints
- compile/check/run/disasm flows
- stable user-facing process surface

Current state:

- strongest current tooling layer
- already part of the public release story

## Layer 2: Formatter

Proposed tool:

- `smfmt` or formatter subcommand under `smc`

Purpose:

- canonical source formatting
- predictable whitespace/layout normalization
- easier code review and example consistency

Current state:

- missing as a public contract

First expectation:

- formatter should normalize Rust-like source consistently
- Logos indentation rules should be formatted intentionally, not left to manual
  editing accidents

## Layer 3: Editor And Language Server

Proposed tool:

- `smlsp`

Purpose:

- diagnostics in editor
- hover/signature help
- go-to-definition for source/module links
- formatting integration

Current state:

- no public editor protocol layer yet

First expectation:

- source diagnostics from parser/type/module layers should become available to
  editor tooling through one canonical server surface

## Layer 4: Debugger And Trace Tooling

Current partial tools:

- `svm disasm`
- verified-path smoke scenarios
- release asset smoke matrix

Purpose:

- inspect source-to-bytecode behavior
- inspect runtime execution shape
- expose trace/debug workflows without requiring users to reverse-engineer the
  VM manually

Current state:

- partial and inspection-oriented
- not yet a full source-level debugger

First expectation:

- debugger maturity should begin with trace and inspection workflows before
  pretending there is a rich interactive debugger

## Layer 5: Docs Generation

Proposed tool:

- `smdoc` or docs subcommand under `smc`

Purpose:

- generate package/module/API documentation from source and package metadata
- distinguish executable docs from design docs

Current state:

- missing as a public tool contract

## Layer 6: Project Scaffolding

Proposed tool:

- `smc init`
- `smc new`

Purpose:

- bootstrap new packages/projects
- create canonical directory layout
- generate manifest and starter sources

Current state:

- missing as a public tool contract

## Layer 7: Test And Profile Helpers

Current partial tools:

- `cargo test`
- golden tests
- smoke matrices
- release bundle verification

Proposed public direction:

- project-aware test runner surface
- profile-aware execution helpers
- performance and quota-aware inspection flows

Current state:

- repository-centric rather than user-centric

## Responsibility Rules

Current intended ownership split:

- CLI is the public shell for compile/run/disasm flows
- formatter owns source layout normalization
- LSP owns interactive editor protocol
- debugger/trace tooling owns execution inspection
- docs tooling owns generated reference output
- scaffolding owns new-project bootstrap

One tool may host multiple subcommands, but these responsibilities should stay
distinct in the design.

## Cross-References

This layer map works together with:

- `docs/roadmap/language_maturity/tooling_maturity.md`
- `docs/roadmap/language_maturity/tooling_workflows.md`
