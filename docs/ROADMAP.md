# Semantic Bootstrap Roadmap

Status: initial roadmap

This roadmap describes migration order, not release dates. A later slice may begin before an earlier one is globally complete when their contracts are independent and stable.

## B0 — Foundation

Goal: establish the first Semantic-owned deterministic substrate.

Candidate scope:

- primitive Semantic value vocabulary needed by bootstrap code;
- quad / four-valued operations used by language semantics;
- stable scalar/value operations;
- deterministic equality/order/hash rules where required;
- source-independent utility structures that do not depend on parser, verifier, VM, host effects, or UI.

Qualification target:

- exact/reference-vector equivalence;
- exhaustive tests where state spaces are small enough;
- mutation proof that the differential gate detects incorrect semantics.

Exit: first `QUALIFIED` Semantic implementation module.

## B1 — Lexical layer

Goal: reproduce tokenization and source-position behavior for an agreed source subset.

Qualification target:

- token kind/value identity;
- span/source-position identity;
- malformed-input behavior for the admitted subset.

## B2 — Parser / AST

Goal: parse an explicitly frozen source subset.

Qualification target:

- normalized AST equivalence;
- parse diagnostic equivalence where contractual;
- no bootstrap-only grammar extensions.

## B3 — Semantic analysis

Goal: reproduce type representation, name resolution, and semantic diagnostics slice by slice.

Qualification target:

- type facts;
- symbol bindings;
- diagnostic classes and relevant payloads;
- deterministic handling of negative cases.

## B4 — IR / lowering

Goal: reproduce canonical lowered meaning independently of Rust implementation details.

Qualification target:

- normalized IR equivalence;
- control/data-flow invariants;
- no frontend/runtime ownership leakage.

## B5 — SemCode emission

Goal: produce canonical SemCode from qualified IR.

Qualification target:

- byte-for-byte identity where the reference emitter defines canonical bytes;
- exact header/revision/capability behavior;
- negative structural cases covered by downstream verifier tests.

## B6 — Verifier

Goal: reproduce the admitted SemCode contract.

This phase starts only for verifier areas whose contracts are sufficiently stable upstream.

Qualification target:

- accept/reject equivalence;
- diagnostic-code equivalence where contractual;
- adversarial malformed-artifact corpus;
- no fail-open fallback paths.

## B7 — VM / runtime

Goal: reproduce deterministic execution semantics for qualified verified programs.

Qualification target:

- value/result equivalence;
- trap/error equivalence where contractual;
- quota and boundary behavior;
- deterministic traces where part of the contract;
- explicit host-effect boundary rather than hidden platform behavior.

## B8 — Self-hosting qualification

Goal: demonstrate a closed bootstrap loop.

Candidate milestone shape:

```text
Semantic compiler source
        ↓
bootstrap compiler
        ↓
SemCode/toolchain artifact
        ↓
qualified execution
        ↓
rebuild / reproduce required compiler components
```

Self-hosting is declared only after the exact reproduction boundary and remaining Foundation dependencies are documented.

## Explicitly outside the critical path

- native Semantic UI implementation;
- Workbench visual layer;
- renderer/layout/component frameworks;
- unrelated application tooling;
- cosmetic parity with the Rust repository.

These may exist independently without blocking language bootstrap.
