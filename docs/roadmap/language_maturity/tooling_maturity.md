# Tooling Maturity

Status: proposed

## Goal

Define the tooling bar required for Semantic to feel like a usable language platform rather than only a compiler-plus-VM stack.

## Why

Rust and Python both feel complete because their tooling is predictable, discoverable, and deeply integrated into everyday development.

## Scope

- formatter expectations
- language-server and IDE integration expectations
- debugger and trace tooling expectations
- docs generation and project scaffolding expectations
- testing and profiling workflow expectations

## Non-Goals

- promising every tool in one release cycle
- treating internal scripts as if they were already public tooling contracts
- widening the core language only to justify tooling work

## Acceptance Criteria

- one canonical tooling roadmap exists
- CLI, formatter, language server, and debugger responsibilities are separated clearly
- at least one editor-facing integration path is documented
- user-facing tooling commands are described as stable, draft, or experimental rather than left implicit
