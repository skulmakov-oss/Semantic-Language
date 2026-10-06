# Semantic Bootstrap Roadmap

Status: qualification-governed bootstrap roadmap

This roadmap describes migration order, not release dates. A later slice may begin before an earlier one is globally complete when their contracts are independent and stable.

Normative companion documents:

- [BOOTSTRAP.md](./BOOTSTRAP.md) — slice lifecycle and differential qualification method;
- [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md) — frozen source subset authority for B1–B5;
- [HOST_CAPABILITY_ABI.md](./HOST_CAPABILITY_ABI.md) — explicit host-effect / capability boundary used by bootstrap tooling and later runtime work.

A slice that depends on one of these contracts must stop when the relevant contract is not frozen enough to support deterministic comparison.

## B0 — Foundation

Goal: establish the first Semantic-owned deterministic substrate.

Candidate scope:

- primitive Semantic value vocabulary needed by bootstrap code;
- quad / four-valued operations used by language semantics;
- stable scalar/value operations;
- deterministic equality/order/hash rules where required;
- source-independent utility structures that do not depend on parser, verifier, VM, host effects, or UI.

Recommended first qualification contour:

```text
quad
  ↓
scalar/value exactness
  ↓
deterministic equality/order/hash
  ↓
differential vectors
  ↓
mutation proof
  ↓
QUALIFIED
```

Qualification target:

- exact/reference-vector equivalence;
- exhaustive tests where state spaces are small enough;
- mutation proof that the differential gate detects incorrect semantics.

Exit: first `QUALIFIED` Semantic implementation module.

B0 Foundation semantics remain host-effect free. Any host interaction needed by compiler tooling must cross the explicit contract in [HOST_CAPABILITY_ABI.md](./HOST_CAPABILITY_ABI.md); it must not leak into Foundation semantics.

## B1 — Lexical layer

Goal: reproduce tokenization and source-position behavior for the source subset frozen in [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md).

Qualification target:

- token kind/value identity;
- span/source-position identity;
- malformed-input behavior for the admitted subset;
- no token or lexical feature outside the frozen subset may become bootstrap-required by implementation accident.

## B2 — Parser / AST

Goal: parse the explicitly frozen source subset defined by [BOOTSTRAP_SUBSET.md](./BOOTSTRAP_SUBSET.md).

Qualification target:

- normalized AST equivalence;
- parse diagnostic equivalence where contractual;
- no bootstrap-only grammar extensions;
- every admitted syntax construction is traceable to the frozen subset authority.

## B3 — Semantic analysis

Goal: reproduce type representation, name resolution, and semantic diagnostics slice by slice for the same frozen subset.

Qualification target:

- type facts;
- symbol bindings;
- diagnostic classes and relevant payloads;
- deterministic handling of negative cases;
- no semantic rule may enter the bootstrap path unless it is admitted by the frozen subset and its reference authority.

## B4 — IR / lowering

Goal: reproduce canonical lowered meaning independently of Rust implementation details.

Qualification target:

- normalized IR equivalence;
- control/data-flow invariants;
- no frontend/runtime ownership leakage;
- lowering input remains bounded by the frozen source subset until an explicit subset revision is accepted.

## B5 — SemCode emission

Goal: produce canonical SemCode from qualified IR.

Qualification target:

- byte-for-byte identity where the reference emitter defines canonical bytes;
- exact header/revision/capability behavior;
- negative structural cases covered by downstream verifier tests.

Canonical SemCode emission must not depend on platform- or process-incidental state.

Forbidden observable inputs include:

- wall-clock time;
- filesystem enumeration order;
- absolute host paths unless the format contract explicitly requires a normalized path representation;
- randomized hash-map / hash-set iteration order;
- process identity;
- thread scheduling;
- locale;
- unspecified map/set order;
- undeclared environment variables.

Any observable constant, symbol, function, section, relocation, or equivalent table must have a contractually defined canonical ordering or another equivalently deterministic construction rule.

## B6 — Verifier

Goal: reproduce the admitted SemCode contract.

This phase starts only for verifier areas whose contracts are sufficiently stable upstream.

Qualification target:

- accept/reject equivalence;
- diagnostic-code equivalence where contractual;
- adversarial malformed-artifact corpus;
- no fail-open fallback paths.

Any verifier reuse/caching introduced later must remain fail-closed and is governed by the stricter admission rules in the self-hosting/Instant Pipeline milestone.

## B7 — VM / runtime

Goal: reproduce deterministic execution semantics for qualified verified programs.

Qualification target:

- value/result equivalence;
- trap/error equivalence where contractual;
- quota and boundary behavior;
- deterministic traces where part of the contract;
- explicit host-effect boundary rather than hidden platform behavior.

Host effects are not implicit VM behavior. File/source access, artifact output, external environment access, and any other platform interaction must cross the declared [HOST_CAPABILITY_ABI.md](./HOST_CAPABILITY_ABI.md) boundary or another explicitly versioned successor contract.

## B8 — Self-hosting qualification

Goal: demonstrate a closed bootstrap loop and a compiler fixed point.

Define:

```text
C0 = reference Rust-hosted Semantic compiler
S  = frozen Semantic source of the bootstrap compiler

C1 = C0(S)
C2 = C1(S)
```

Required convergence criterion:

```text
Canonical(C1.smc) == Canonical(C2.smc)
```

Prefer the stronger result when the SemCode contract permits it:

```text
C1.smc == C2.smc
```

Candidate execution shape:

```text
Semantic compiler source S
        ↓
reference compiler C0
        ↓
C1
        ↓
C1 compiles S
        ↓
C2
        ↓
canonical fixed-point comparison
        ↓
Bootstrap Seal
```

Self-hosting is declared only when:

- the fixed-point rule passes with zero unexplained deltas;
- the exact reproduction boundary is recorded;
- remaining Foundation/host dependencies are documented;
- no hidden host parser/typechecker/lowering/emitter fallback exists in the qualification path;
- the Bootstrap Seal captures the provenance required by the self-hosting milestone.

## Explicitly outside the critical path

- native Semantic UI implementation;
- Workbench visual layer;
- renderer/layout/component frameworks;
- unrelated application tooling;
- cosmetic parity with the Rust repository.

These may exist independently without blocking language bootstrap. TypeScript Workbench/Studio remains a presentation consumer of Semantic-owned contracts and is governed separately by the Semantic Reactive Interface milestone.
