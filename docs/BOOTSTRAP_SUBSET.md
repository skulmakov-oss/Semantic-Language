# Semantic Bootstrap Source Subset

Status: **NOT YET FROZEN**

This document is the normative registry for the source-language subset admitted into the first Semantic self-hosting path.

It exists to prevent bootstrap scope from expanding implicitly while lexer, parser, semantic analysis, lowering, and emission are being implemented.

## 1. Authority

B1–B5 in [ROADMAP.md](./ROADMAP.md) may claim a source construction only when that construction is explicitly listed here as **ADMITTED** and its reference authority is identified.

This document does not redefine the Semantic language.

Until ownership transfer, the Rust-hosted reference implementation and the applicable normative language/specification documents remain the semantic oracle.

If this document conflicts with an upstream authoritative contract, the affected bootstrap slice stops until the conflict is resolved explicitly.

## 2. State model

Every candidate construction has one of these states:

```text
UNLISTED
   ↓
CANDIDATE
   ↓
ADMITTED
   ↓
FROZEN
```

Definitions:

- **UNLISTED** — not part of the bootstrap subset;
- **CANDIDATE** — being evaluated; must not be required by the qualification path;
- **ADMITTED** — approved for implementation/differential work with identified authority;
- **FROZEN** — admitted behavior and qualification form are stable enough for bootstrap dependency.

Removing or semantically changing a FROZEN construction requires an explicit contract decision and requalification of affected slices.

## 3. Required registry fields

Every admitted construction must record:

| Field | Meaning |
|---|---|
| ID | stable bootstrap-subset identifier |
| Construction | language/source feature name |
| State | CANDIDATE / ADMITTED / FROZEN |
| Reference authority | normative spec/path/contract |
| Required by | compiler component or bootstrap source location/class |
| Lexer evidence | token/span comparison form |
| Parser evidence | normalized AST comparison form |
| Sema evidence | type/binding/diagnostic comparison form |
| Lowering evidence | normalized IR comparison form where applicable |
| Negative cases | malformed/invalid forms that must be compared |
| Notes | explicit limits or exclusions |

## 4. Frozen-subset rule

The first bootstrap source set must be the **smallest subset that can express the compiler-domain code needed for the C0 → C1 → C2 proof**.

Convenience is not sufficient reason to enlarge it.

A proposed addition must answer:

1. Which compiler-domain requirement cannot be expressed without it?
2. What is the current reference authority?
3. Which B1–B5 qualification outputs are affected?
4. Which existing frozen cases must be re-run?
5. Does the addition introduce host effects, nondeterminism, or a new runtime dependency?

If these questions are not answered, the feature remains CANDIDATE.

## 5. Initial registry

No source construction is declared FROZEN by this document yet.

That is intentional: the current repository documents describe the bootstrap process but do not enumerate enough concrete grammar/type constructions to create an honest frozen list without importing assumptions.

The first SH-00 / bootstrap-authority work must populate this registry from the current reference grammar/spec authority before B1/B2 qualification is treated as frozen.

| ID | Construction | State | Reference authority | Required by | Notes |
|---|---|---|---|---|---|
| BS-000 | bootstrap source subset baseline | CANDIDATE | to be pinned by SH-00 authority manifest | C0 → C1 → C2 compiler source | placeholder authority row; not a language feature |

## 6. Explicit exclusions

Unless later admitted through the registry, the first bootstrap path must not silently require:

- bootstrap-only grammar extensions;
- syntax accepted only by the new implementation;
- convenience constructs with no frozen reference behavior;
- UI-specific language features;
- host/platform-specific syntax;
- nondeterministic or environment-dependent source behavior;
- new language design introduced solely to simplify the port.

## 7. Qualification relationship

For each FROZEN construction, evidence must flow through the relevant stages:

```text
source example
   ↓
lexer tokens/spans
   ↓
normalized AST
   ↓
semantic facts/diagnostics
   ↓
normalized IR
   ↓
canonical SemCode
```

Not every construction needs a unique assertion at every stage, but every observable contract it influences must have a declared comparison form.

Negative and malformed cases are part of the subset contract where the reference behavior is contractual.

## 8. Freeze procedure

The subset is considered frozen for a bootstrap slice only when:

- all constructions required by that slice are listed;
- their reference authority is pinned;
- their state is FROZEN;
- positive and negative comparison forms are specified;
- no unresolved authority conflict affects them;
- the subset revision/commit identity is recorded in qualification evidence.

## 9. Change control

A change to the frozen subset is not an ordinary parser implementation detail.

It is a contract change.

The change record must include:

- reason;
- affected construction IDs;
- previous and new authority;
- affected bootstrap slices;
- evidence invalidated by the change;
- required requalification.

The objective is to keep the bootstrap source language deliberately boring, explicit, and reproducible until the fixed point is sealed.
