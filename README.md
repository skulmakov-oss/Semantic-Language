# Semantic Language

> **Bootstrap repository for the Semantic programming language.**

This repository is the migration path from the current Rust-hosted Semantic implementation to a progressively self-hosted Semantic toolchain.

The existing [`skulmakov-oss/Semantic`](https://github.com/skulmakov-oss/Semantic) repository remains the **reference implementation and behavioral oracle** during bootstrap.

## Status

**Early bootstrap / architecture establishment.**

This repository is not yet the canonical implementation of Semantic and must not silently redefine language semantics, SemCode, verifier behavior, or runtime contracts.

## Goal

The long-term target is:

```text
Semantic source
      ↓
Semantic compiler written substantially in Semantic
      ↓
SemCode
      ↓
Semantic verifier/runtime
      ↓
Semantic builds Semantic
```

Rust remains the historical Foundation and reference implementation until each migrated module has sufficient differential evidence to replace it.

## Bootstrap strategy

Bootstrap proceeds **module by module**, not as a rewrite.

```text
Reference Semantic (Rust)
        │
        ├──────────────┐
        ▼              ▼
   reference path   bootstrap path
        │              │
        └──────┬───────┘
               ▼
        differential proof
               ▼
          qualified module
```

A module may enter bootstrap when its observable contract is stable enough to compare against the reference implementation. The entire issue backlog of the reference repository does **not** need to be closed first.

## Core rules

1. **Reference first.** Existing Semantic behavior is the oracle unless an explicit upstream design decision changes the contract.
2. **No silent divergence.** Bootstrap code must not invent new semantics merely to simplify migration.
3. **Differential qualification.** Equivalent inputs should produce equivalent observable results across reference and bootstrap implementations.
4. **Freeze locally.** A completed bootstrap slice freezes only the contract it actually proves.
5. **No UI dependency.** Self-hosting the language does not require a native Semantic UI stack. UI belongs outside the bootstrap-critical path.
6. **Rust is not deleted.** Its role evolves from implementation → oracle → bootstrap host → historical Foundation.

## Planned bootstrap contour

The exact dependency order may evolve as the reference architecture changes, but the initial direction is:

```text
B0  Foundation primitives and deterministic semantic values
B1  Lexical layer and source positions
B2  Parser / AST subset
B3  Semantic analysis and type representation
B4  IR model and lowering subset
B5  SemCode emission
B6  Verifier
B7  VM / runtime execution
B8  Self-hosting qualification
```

Hot or actively changing contracts may remain reference-only while independent lower-risk slices continue.

## What blocks a bootstrap slice?

A finding blocks a slice when it can change one of these:

- Semantic meaning;
- the module's public contract;
- serialized / wire representation;
- deterministic observable behavior.

Documentation debt, UI work, unrelated tooling, and peripheral issues do not automatically block independent bootstrap work.

## Repository layout

```text
docs/
  ARCHITECTURE.md
  BOOTSTRAP.md
  ROADMAP.md
bootstrap/
  # Semantic bootstrap modules will land here incrementally
qualification/
  # differential and bootstrap-specific gates
reference/
  # adapters/manifests describing the Rust reference boundary
```

Directories are introduced only when their first real implementation slice lands; this repository intentionally starts documentation-first rather than pre-creating empty architecture.

## Relationship to Semantic

| Repository | Role |
| --- | --- |
| [`Semantic`](https://github.com/skulmakov-oss/Semantic) | Current canonical Rust-hosted implementation and oracle |
| **Semantic-Language** | Bootstrap implementation and future self-hosted language repository |

Until an explicit migration milestone says otherwise, disagreement is treated as a bootstrap finding — not as permission for this repository to redefine the reference behavior.

## License

Apache License 2.0. See [`LICENSE`](LICENSE).
