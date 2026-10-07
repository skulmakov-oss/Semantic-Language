# Bootstrap Source Subset

Status: **initial registry populated by SHF-0 (issue #11) — no construction FROZEN yet**

Salvaged from PR #6 (`docs/BOOTSTRAP_SUBSET.md` @ `3714ce27`) and re-owned under the SHF
architecture.

This document defines the **minimum Semantic source subset needed to express compiler source
`S`**. It is a registry and change-control contract. It does not define the Semantic language:
language semantics remain owned by `skulmakov-oss/Semantic` (see [OWNERSHIP.md](./OWNERSHIP.md)).

## 1. Authority

- A source construction may be required by `S` only when it is listed here as `ADMITTED` or
  `FROZEN` with an identified reference authority at an exact SHA.
- If this registry conflicts with an upstream contract, the affected work stops until the
  conflict is resolved explicitly upstream.
- Feature IDs (`BSF-nnn`) are local contract identifiers. They are not milestones and do not
  form a roadmap; the only milestone vocabulary is SHF-0 … SHF-17.

## 2. State model

```text
UNLISTED -> CANDIDATE -> ADMITTED -> FROZEN
```

- **UNLISTED** — not part of the bootstrap subset.
- **CANDIDATE** — under evaluation; must not be required by any qualification path.
- **ADMITTED** — approved for compiler use and differential work, with identified authority.
- **FROZEN** — behavior and qualification form are stable enough for bootstrap dependency.

Removing or changing a `FROZEN` construction is a contract change that requires requalification
of every affected compiler stage.

## 3. Minimality rule

The subset is the **smallest** set of constructions that can express `S`. Convenience is not
a reason to admit a construction. A construction cannot enter the bootstrap-required subset
merely because it is easy to use.

## 4. Admission requirements

Every `ADMITTED` or `FROZEN` entry must record:

| Field | Meaning |
|---|---|
| ID | local identifier `BSF-nnn` |
| Construction | language/source feature |
| State | CANDIDATE / ADMITTED / FROZEN |
| Reference authority | upstream spec path + exact SHA |
| Required compiler use | which part of `S` cannot be expressed without it |
| Positive evidence | comparison form for valid uses |
| Negative evidence | malformed/invalid forms that must be compared |
| Affected compiler stages | lexer / parser / sema / IR / emitter (SHF-10 … SHF-14) |
| Requalification impact | evidence invalidated if this entry changes |

An entry missing any field stays `CANDIDATE`.

## 5. Registry

Authority for every entry: `skulmakov-oss/Semantic@89641da8237f4fcefb50cf1958a50e4d4003aea7`
(C0), `docs/spec/foundation_source_profile_v1.md` — contract `semantic.foundation.source/1.2`,
section "Included executable surface", with executable evidence mapped in
`docs/roadmap/stable_foundation/stable_public_language_contract.md`. An entry is `ADMITTED`
only where that section lists the construction as Included; nothing is `FROZEN` until its
comparison forms run against C0 in the owning SHF stage. Machine-readable state lives in
[`bootstrap/contract.toml`](../bootstrap/contract.toml) `[subset]`; the validator rejects any
mismatch with this table.

Evidence forms (used by the columns below): **T** tokens + spans, **A** normalized AST,
**F** semantic facts + diagnostics, **I** normalized IR, **B** SemCode bytes vs C0.
"Stages" names the SHF compiler stages whose evidence an entry change invalidates.

### Admitted (Included at C0)

| ID | Construction | State | Required compiler use | Positive / negative evidence | Stages |
|---|---|---|---|---|---|
| BSF-001 | Items and functions: explicit parameter/return types, calls, `fn main()` | ADMITTED | every compiler pass is a function; driver entrypoint | T A F I B / arity, type, unknown-call diagnostics | SHF-10..14 |
| BSF-002 | Bindings: `let`, `const`, `let mut`, assignment, block scope, `return` | ADMITTED | local state in all passes | T A F I B / const, type, unknown-target diagnostics | SHF-10..14 |
| BSF-003 | Control: `if`/`else`, blocks, `while`, `loop`, range `for`, `break`/`continue` | ADMITTED | scanning, worklists, traversals | T A F I B / outside-loop and branch-mismatch diagnostics | SHF-10..14 |
| BSF-004 | `bool` and `i32` with frozen overflow policy | ADMITTED | counters, indices within `i32`, flags | T A F I B / mixed-family, division-by-zero traps | SHF-10..14 |
| BSF-005 | Nominal records: construction, field read, copy-with | ADMITTED | tokens, AST/IR nodes, compiler state | T A F I B / record construction and copy-with diagnostics | SHF-10..14 |
| BSF-006 | Enums/ADTs, `Option`, `Result`, exhaustive `match` on admitted scrutinees | ADMITTED | token kinds, AST variants, error propagation | T A F I B / exhaustiveness, or-pattern and range-lowering rejections | SHF-10..14 |
| BSF-007 | `Sequence(T)`: values, indexing, iteration, length, push/pop | ADMITTED | token streams, node lists, emitted code buffers | T A F I B / collection type diagnostics | SHF-10..14 |
| BSF-008 | `Map(K, V)`: empty, get, set, contains (no traversal) | ADMITTED | symbol tables by key lookup | T A F I B / key/value type diagnostics | SHF-12..14 |
| BSF-009 | `text`: UTF-8 literals, equality, concatenation, `to_text` | ADMITTED | identifiers and diagnostic messages | T A F I B / indexing and cross-family negatives | SHF-10..14 |
| BSF-010 | Direct local-path bare and selected imports; missing-file, duplicate and cycle rejection | ADMITTED | splitting `S` into modules | T A F I B / alias, wildcard, re-export, cycle negatives | SHF-11..12 |

### Candidates (capability gaps owned upstream)

A candidate must not be required by any qualification path until upstream delivers it in its
SHF capability stage (SHF-1 … SHF-9) and this registry admits it with authority at a newly qualified C0.

| ID | Construction | State | Gap at C0 | Owner / stage |
|---|---|---|---|---|
| BSF-101 | `text` byte inspection, slicing and search | CANDIDATE | indexing/slicing not Included | `skulmakov-oss/Semantic` / SHF-1 |
| BSF-102 | Byte values, binary buffers, binary file output | CANDIDATE | only UTF-8 text I/O exists | `skulmakov-oss/Semantic` / SHF-2 |
| BSF-103 | `u32` arithmetic, bit operations, conversions | CANDIDATE | `u32` arithmetic deferred (SSF-07) | `skulmakov-oss/Semantic` / SHF-3 |
| BSF-104 | Deterministic `Map`/`Set` traversal and `Set` | CANDIDATE | traversal order not in contract | `skulmakov-oss/Semantic` / SHF-4 |
| BSF-105 | Executable generics / monomorphisation | CANDIDATE | broad generics experimental | `skulmakov-oss/Semantic` / SHF-5 |
| BSF-106 | Module exports and project resolution for a multi-module `S` | CANDIDATE | exports/namespace access not Included | `skulmakov-oss/Semantic` / SHF-7 |

## 6. Exclusions

Unless admitted through §4, `S` must not require:

- bootstrap-only grammar or semantics;
- syntax accepted only by the bootstrap compiler;
- constructs with no reference behavior;
- UI-specific language features;
- host/platform-specific syntax;
- nondeterministic or environment-dependent source behavior;
- language design introduced to simplify the port.

## 7. Qualification relationship

For each `FROZEN` construction, every observable contract it influences has a declared
comparison form along the compiler path:

```text
source -> tokens/spans -> normalized AST -> semantic facts/diagnostics -> normalized IR -> canonical SemCode
```

## 8. Change control

A change record states: reason, affected IDs, previous and new authority (with SHA), affected
SHF stages, invalidated evidence, and required requalification.
