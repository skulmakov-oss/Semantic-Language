# Contributing to Semantic-Language

This repository is a bootstrap and self-hosting project. Changes are evaluated first by contract preservation, not by amount of code ported.

## Before implementation

For every non-trivial bootstrap change, identify:

1. the reference contract being mirrored;
2. its current owner in `skulmakov-oss/Semantic`;
3. the observable input/output boundary;
4. known upstream issues that can invalidate the slice;
5. the intended differential comparison;
6. the exact claim the completed slice will prove.

If any of these are unknown and materially affect semantics, investigate before widening implementation scope.

## Pull request shape

Prefer narrow PRs that contain one migration or qualification unit.

A good bootstrap PR explains:

- **Reference point** — upstream commit/ref and contract owner;
- **Purpose** — what is being mirrored or qualified;
- **Scope** — what is deliberately not included;
- **Equivalence rule** — how Rust/reference and Semantic/bootstrap outputs are compared;
- **Regression evidence** — positive, negative, and boundary cases;
- **Mutation/adversarial evidence** — proof that the qualification would fail for a meaningful defect;
- **Remaining dependencies** — what still prevents promotion to the next migration state.

## Migration state

Do not call a module canonical merely because it compiles.

Use the repository state model:

```text
REFERENCE_ONLY → MIRRORED → DIFFERENTIAL → QUALIFIED → CANONICAL
```

`CANONICAL` requires an explicit ownership-transfer decision.

## No silent language design

Bootstrap work must not silently introduce:

- new syntax;
- new type semantics;
- new SemCode encodings;
- relaxed verifier admission;
- new runtime fallback behavior;
- new host effects;
- UI requirements.

If a migration exposes a flaw or desirable language change, handle it as an explicit contract decision in the reference architecture first, then update bootstrap evidence.

## Qualification principle

Prefer exactness over convenience.

Where byte identity is the contract, compare bytes. Where structured identity is the contract, compare normalized structure. Do not replace a strong comparison with a weaker smoke test simply to get green CI.

## Rust reference

The Rust-hosted Semantic implementation is not disposable scaffolding during early bootstrap. It is the behavioral oracle and historical Foundation.

Removing a reference path is a later migration decision, not routine cleanup.

## UI boundary

Native UI, renderer, layout, components, and Workbench presentation are outside the language self-hosting critical path unless an explicit future architecture decision changes that boundary.
