# Tooling Workflows

Status: proposed v0

## Purpose

This document defines the canonical user workflows that the Semantic tooling
stack should eventually support cleanly.

It also distinguishes current workflows from proposed ones, so the repository
does not overclaim maturity it does not yet have.

## Stability Labels

Current workflow labels:

- `stable now`
- `draft target`
- `experimental idea`

## Workflow 1: Compile / Run / Disasm

Status: `stable now`

Current path:

- `smc compile`
- `smc run`
- `svm run`
- `svm disasm`

Why it matters:

- this is the current strongest user-facing loop
- it already anchors release smoke tests and examples

## Workflow 2: Check And Diagnostics In Editor

Status: `draft target`

Target path:

- editor sends source to `smlsp`
- parser/type/module diagnostics surface inline
- go-to-definition follows local and imported symbols

Why it matters:

- this is the minimum path to a modern authoring loop

## Workflow 3: Format Source

Status: `draft target`

Target path:

- `smc fmt <file.sm>`
- `smc fmt --check <file.sm>`

Why it matters:

- users need one canonical source layout
- examples and docs should stop depending on manual formatting

Canonical rule:

- `smc fmt` is the first public formatter surface
- any later `smfmt` wrapper should remain a thin convenience shell around the
  same formatting contract

## Workflow 4: Debug And Trace

Status: `draft target`

Target path:

- compile with debug-oriented metadata where allowed
- inspect disassembly and execution trace together
- eventually bridge source spans to SemCode offsets

Current honest baseline:

- `svm disasm` exists
- richer source-level debugging does not yet exist as a public workflow

## Workflow 5: Generate Documentation

Status: `draft target`

Target path:

- generate package/module/API docs from source and package metadata
- expose importable library surface, examples, and compatibility notes

Why it matters:

- a richer language cannot rely forever on hand-written repository docs only

## Workflow 6: Create New Project

Status: `draft target`

Target path:

- `smc new package_name`
- `smc init`

Expected result:

- canonical package structure
- manifest
- starter sources
- example test or smoke file

## Workflow 7: Test And Profile

Status: `experimental idea`

Target path:

- package-aware test command
- profile-aware run/check helpers
- quota/debug-oriented execution insights

Current honest baseline:

- repository maintainers can run cargo-based tests and release smoke checks
- ordinary users do not yet have a mature language-native test workflow

## Immediate Priority Order

The first tooling maturity wave should prioritize:

1. compile/run/disasm stability
2. formatter
3. editor/LSP diagnostics
4. debug/trace workflows
5. docs generation
6. scaffolding

This order keeps the authoring loop ahead of nicer-to-have platform polish.

## Cross-References

This workflow map depends on:

- `docs/roadmap/language_maturity/tooling_maturity.md`
- `docs/roadmap/language_maturity/tooling_layers.md`
- `docs/roadmap/language_maturity/formatter_contract.md`
