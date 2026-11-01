# Contributing to Semantic-Language

This repository is a bootstrap and self-hosting project. Changes are evaluated first by contract preservation, not by amount of code ported.

## Primary engineering rule — Semantic Capability-First

**The implementation MUST exploit all applicable, admitted capabilities of Semantic.**
Design compiler logic as native Semantic, not as a translation of conventional Boolean-only,
Rust, Python or C architecture that discards Semantic's expressive model.

Before significant design or implementation changes, identify the exact pinned-C0 source
capabilities relevant to the task (types, quad operators, pattern matching, deterministic
collections, records/ADTs, control flow, text/binary support, or other admitted facilities).
Use them directly when they express the intended semantics. Do not replace supported
language constructs with custom emulation, lossy `bool` flags, hidden host/compiler logic
or redundant encodings merely because those patterns are familiar.

- **Evidence is not a Boolean:** when a proposition can have positive, negative, missing or
  conflicting witnesses, preserve N/F/T/S; distinguish knowledge and truth operations,
  retain proposition/witness origins, and never treat N as F or S as unconditional T.
  Evidence state does not itself establish provenance authenticity or authorization.
- **Binary facts stay binary:** membership, lexical mode and structural validity may be
  correctly represented with `bool`. Do not fabricate quad witnesses or replace every
  `bool` for cosmetic consistency.
- **No invented capability:** a Rust-only primitive, design proposal or experimental
  feature is not automatically available to the pinned Semantic source path. If a
  necessary capability is missing, record the gap and qualify it upstream under separate
  owner authorization; do not patch it through private bootstrap host behavior.
- **Prove the choice:** for meaningful changes, record the native features considered,
  the semantic role they fill, any deliberate exception, and positive/negative/boundary
  tests against the exact pinned C0 with verifier/VM admission as applicable.

"Full use" means **no artificial restriction of Semantic's applicable expressiveness**,
not forcing unrelated features into a function. It never overrides canonical specifications,
bootstrap quotas, the frozen C0 pin, the active Harness, or separate owner decisions.
See [the canonical architecture invariant](docs/ARCHITECTURE.md#31-semantic-capability-first-invariant).

## Before implementation

For every non-trivial bootstrap change, identify:

1. the SHF stage it belongs to ([docs/ROADMAP.md](docs/ROADMAP.md)) and its owner repository ([docs/OWNERSHIP.md](docs/OWNERSHIP.md));
2. the reference contract being mirrored;
3. the frozen reference specification in `skulmakov-oss/Semantic`;
4. the observable input/output boundary;
5. the intended differential comparison;
6. the exact claim the completed slice will prove.

Distinguish technical contract dependencies from administrative repository dependencies:
downstream bootstrap implementation depends on frozen normative contracts and exact reference SHAs,
never on live upstream GitHub issue or PR lifecycles.

If any of these are unknown and materially affect semantics, investigate before widening implementation scope.

## Harness workflow

Every change runs under the active task envelope `.harness/current.task.yaml` (rules: `AGENTS.md`).

- **Before work:** read the envelope, confirm the task and its authority, confirm every planned path fits `allowed_paths` and none hits `forbidden_paths`.
- **Before commit:** `pwsh -File scripts/harness-check.ps1` and `git diff --check`.
- **Before PR handoff:** `pwsh -File scripts/harness-check.ps1 -BaseRef <exact-base-sha>`, `git diff --name-only <base>...HEAD`, `git status`.

Ordinary SHF tasks work within the stable engineering boundary without recurring envelope transition PRs.
A task that needs changes to protected governance surfaces stops and asks the owner for a governance transition.

## Pull request shape

Prefer focused PRs that deliver complete stage units under an authorized SHF envelope.
Under Strategy v1.1, ordinary SHF tasks may deliver implementation, tests, and documentation
in a unified PR cycle, eliminating artificial multi-PR fragmentation while maintaining strict
fail-closed governance boundaries.

A good bootstrap PR explains:

- **Reference point** — upstream exact SHA (never floating `main`), treated as a reproducible historical executable reference;
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

Prefer exactness over convenience. The full discipline is in [docs/QUALIFICATION.md](docs/QUALIFICATION.md).

Where byte identity is the contract, compare bytes. Where structured identity is the contract, compare normalized structure. Do not replace a strong comparison with a weaker smoke test simply to get green CI.

## Rust reference

The Rust-hosted Semantic implementation is not disposable scaffolding during early bootstrap. It is the behavioral oracle and historical Foundation.

Removing a reference path is a later migration decision, not routine cleanup.

## UI boundary

Native UI, renderer, layout, components, and Workbench presentation are outside the language self-hosting critical path unless an explicit future architecture decision changes that boundary.
