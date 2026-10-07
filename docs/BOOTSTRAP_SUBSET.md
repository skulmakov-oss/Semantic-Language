# Bootstrap Source Subset

Status: **NOT YET FROZEN — registry empty by design**

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

No construction is admitted yet. Populating this registry from the pinned reference is part of
SHF-0 and later stages; it must not be done by assumption.

| ID | Construction | State | Reference authority | Required compiler use | Notes |
|---|---|---|---|---|---|
| — | — | — | — | — | empty |

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
