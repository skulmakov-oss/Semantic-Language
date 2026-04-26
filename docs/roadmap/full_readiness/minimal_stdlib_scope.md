# FR-3 Minimal Standard Library Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Define the minimum standard-library baseline required for Semantic to feel usable as a practical language while preserving verifier-first and deterministic runtime discipline.

This document scopes stdlib v0. It does not implement the library.

## Design rules

- Stdlib entries must be explicit, not hidden compiler magic.
- Host-bound operations must remain capability-aware.
- Deterministic helpers must be reproducible by configuration and input.
- Stdlib must not bypass type checking, lowering, SemCode, verifier, or VM contracts.
- The first baseline should be small and boring.

## Candidate module set

### core

Minimum surface:

- `assert` / `debug_assert` policy;
- compare helpers where useful;
- quad helpers where they reduce boilerplate without hiding `quad` semantics.

Acceptance:

- helpers do not change language semantics;
- assertion failure path is deterministic.

### math

Minimum surface:

- basic numeric helpers for admitted number families;
- explicit overflow/trap/non-commitment policy;
- no implicit widening beyond documented type rules.

Acceptance:

- arithmetic helper behavior matches source/type semantics;
- edge cases are tested when implemented.

### text

Minimum surface:

- concatenation or equivalent composition path;
- length/query helpers;
- minimal value-to-text conversion;
- minimal formatting policy.

Acceptance:

- text helpers are sufficient for example output;
- formatting remains deterministic;
- unsupported formatting forms fail clearly.

### seq

Minimum surface:

- length;
- emptiness;
- get/index behavior;
- contains where equality is admitted;
- mutation helpers only if mutation model is admitted.

Acceptance:

- index failure behavior is specified;
- sequence helpers preserve deterministic order.

### map

Minimum surface:

- get;
- set/insert if mutation model is admitted;
- contains;
- remove if admitted;
- deterministic iteration status explicitly classified.

Acceptance:

- key constraints are documented;
- ordering behavior is explicit;
- unsupported key families are rejected deterministically.

### result

Minimum surface:

- helpers for `Option` and `Result` where useful;
- no hidden exception model;
- helper names align with language identity.

Acceptance:

- examples can express ordinary success/failure flow;
- helper behavior does not conflict with match semantics.

### rand

Minimum surface:

- deterministic seeded pseudo-random source;
- no ambient entropy in pure execution;
- replay behavior defined.

Acceptance:

- same seed and same calls produce same sequence;
- host entropy, if ever admitted, is separate and capability-gated.

### io

Minimum surface:

- narrow debug/stdout print path;
- explicit capability/host boundary if host-bound;
- no file/network IO in stdlib v0 unless separately scoped.

Acceptance:

- examples can print/debug results;
- output order is deterministic relative to execution trace.

## Work packages

- FR-3.1 — define stdlib ownership model;
- FR-3.2 — define import path and namespace policy;
- FR-3.3 — scope `core` helpers;
- FR-3.4 — scope `math` helpers;
- FR-3.5 — scope `text` helpers;
- FR-3.6 — scope `seq` helpers;
- FR-3.7 — scope `map` baseline;
- FR-3.8 — scope deterministic `rand`;
- FR-3.9 — scope narrow `io` / debug output;
- FR-3.10 — add stdlib examples and tests plan.

## Out of scope

- package registry;
- broad filesystem/network IO;
- host clock/entropy as implicit sources;
- async/concurrency;
- UI;
- reflection/macros;
- broad generic collection framework unless separately scoped.

## Definition of Done

FR-3 is complete when stdlib v0 has a small documented ownership model, admitted module list, import path, behavior contracts, and follow-up implementation issues with tests for each admitted helper family.
