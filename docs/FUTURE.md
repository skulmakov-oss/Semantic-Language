# Future Work (post-Bootstrap)

Status: deferred. **None of the work below is a prerequisite for C0 → C1 → C2.**

These tracks may only build on proven semantics. They consume the result of the self-hosting
fixed point; they do not define it.

```text
Self-hosting / Bootstrap Seal
        |
        +--> Instant Pipeline
        |
        +--> SRI / TypeScript tooling
        |
        +--> Semantic#1909 Full Sigma + t¤
```

## Instant Pipeline (from Issue #4, deferred part)

Starts after the Bootstrap Seal. Rule: *prove correctness first, freeze the fixed point, then
optimize the proven semantics.* Measurement and disposable research may happen earlier but must
never enter the qualification dependency chain of C1/C2.

- persistent compiler service;
- incremental syntax;
- incremental query / dependency engine;
- incremental IR, SemCode and admission (admission reuse must cover the full verifier/environment
  dependency closure and fail closed on uncertainty);
- SEMIMG prepared execution image with flat deterministic layout;
- near-zero startup runtime;
- clean-build oracle: incremental output must equal a clean rebuild for the same inputs.

## SRI — Semantic Reactive Interface (from Issue #5)

Reactive boundary between Semantic-owned services and TypeScript tooling: generated TypeScript
projections, source/semantic delta protocol, revision and cancellation model, event/query/command
families. TypeScript is a presentation consumer only and **never a language authority**; `quad`
must never collapse to `boolean` across the boundary.

## Generated TypeScript boundary / Workbench / Studio

External presentation clients of stable Semantic service contracts. The native Semantic UI /
Workbench / Studio effort in the reference repository is retired and is not restored here.

## Optional native backend

Acceleration backend only, after canonical self-hosted SemCode compilation is stable. Never a
second semantic authority.

## Semantic#1909 — Native Reasoning / Full Sigma + t¤

Sibling post-self-hosting track upstream. Not part of the bootstrap work queue.

## Verifier or VM in Semantic

A Semantic verifier or VM may be considered after the Seal through the normal migration states
and an explicit ownership-transfer decision. It is not required for first self-hosting.
