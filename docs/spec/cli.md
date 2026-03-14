# CLI Specification

Status: draft v0
Canonical owner crate: `smc-cli`
Current process entrypoints: root `smc` and `svm` binaries

## Purpose

This document defines the current public CLI contract for the Semantic toolchain.

Current owner rule:

- `smc-cli` owns the public CLI contract in the current `v1` baseline
- root `src/bin/smc.rs` and `src/bin/svm.rs` are process entrypoints, not second long-term owners
- the CLI must orchestrate public crate APIs rather than redefine compiler, verifier, VM, or profile semantics

## Current Command Surface

Current command families include:

- `compile`
- `check`
- `lint`
- `watch`
- `verify`
- `run`
- `run-smc`
- `disasm`
- `dump-ast`
- `dump-ir`
- `dump-bytecode`
- `hash-ast`
- `hash-ir`
- `hash-smc`
- `features`
- `doctor`
- `explain`
- `repl`
- `profile show`
- `profile train`
- `profile validate`

This draft does not claim that every command is permanently frozen, but it defines the current public CLI surface that tooling may rely on.

## Contract-Sensitive Commands

The following commands expose contract state rather than only workflow convenience:

- `smc verify`
- `smc profile show`
- `smc profile validate`
- `smc doctor`
- `smc features`

Changes to those commands should be reviewed as public contract changes.

## Output Modes

Current output families:

- human-readable text
- machine-readable JSON for selected contract-heavy commands

Current machine-readable JSON outputs:

- `smc profile show --json`
- `smc doctor --json`

Rule:

- machine-readable output must remain structurally intentional
- changing JSON field names or meaning is a public contract change

## Verified Execution Rule

Current execution-facing commands follow this split:

- `smc run <input.sm>` compiles source and executes
- `smc run-smc <input.smc>` executes compiled SemCode through the verified path
- `smc verify <input.smc>` performs admission without execution

Public rule:

- standard `.smc` execution must not bypass verification

## Exit Behavior

Current rule:

- successful command execution exits successfully
- user-visible contract violations, parsing failures, verification failures, and I/O failures produce non-zero termination through CLI error propagation

This draft does not yet formalize a complete numeric exit-code taxonomy.

## Change Review Rule

A CLI change requires explicit review if it changes:

- command names
- flag names
- JSON field names
- semantics of contract-reporting commands
- verified execution behavior

Such changes should update this specification in the same change series.
