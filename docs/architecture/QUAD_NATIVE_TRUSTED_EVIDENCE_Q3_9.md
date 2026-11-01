# Q3.9 — Canonical Sema Replay Admission (Experimental)

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Category:** source-to-lowering admission security boundary. **Not** a new opaque language-level Evidence type.

## Threat and trusted base

Previously `q3_lowering_error` checked that `Evidence` had a plausible `T` state, matching proposition identity and source byte offset, plus some typed-AST consistency. Because `record Evidence` and `record Sem` have public constructors and `with` updates, a caller could forge both a witness and corroborating semantic metadata. A *structurally valid* T was not authentic evidence of evaluation by Sema.

Q3.9 treats the **existing trusted `analyze(Prog)` implementation** as the construction authority. In the production `compile_source` path, the first `Sem` is a candidate, never an authorization token. Before lowering, `q3_trusted_admit(candidate)` runs `analyze(candidate.prog)` again and obtains a **canonical** `Sem` from the parsed AST.

## Admission algorithm

1. Re-run trusted Sema on the complete AST. If replay reports a diagnostic, reject regardless of a caller's cleared `sem.err` or manufactured `Evidence`.
2. Compare the candidate against the replayed canonical state in `q3_replay_difference`. For each AST node index, compare **presence and complete ordered contents** of the six evidence maps: `type_evidence`, `map_container_evidence`, `map_key_evidence`, `map_value_evidence`, `tuple_shape_evidence`, and `if_expr_condition_evidence`. Valid carriers must agree in proposition kind, owner, scope, derived quad state, complete positive sequence and complete negative sequence.
3. Compare AST-indexed semantic metadata maps `exprty`, `slots`, `callees`, `fieldix`, `tuple_expected`, `inferred_binding_types`, `tuple_inferred_components`, `match_result_checked`. Also compare complete type-table shapes/elements, arithmetic mode, function return and parameter type IDs and local symbol names.
4. Run existing `q3_lowering_error` on the **replayed** `Sem`, retaining exact fail-closed type/ownership precursor checks already in the experimental path.
5. Return `TrustedSemAdmission { ok, canonical, error }`. The production driver **must pass only `admission.canonical`** into `lower_function` and use its arithmetic mode during encoding. Never lower the original candidate record.

The replay comparison identifies divergence, while canonical reconstruction discards all candidate-supplied noncanonical carriers before code generation. This moves authority from self-declared evidence fields to the actual trusted type-checking computation.

## Regression test and tamper attacks

`tests/compiler/programs/quad_trusted_admission_internal.sm` imports actual `lower.sm`, replays a valid source containing map, tuple inference, scalar inference, operators and control types, then checks:
- a fully valid local `TrustedSemAdmission` and deterministic replay;
- a forged but structurally valid `T` witness with an invalid source position;
- altered inferred binding type and altered expression inferred type;
- substituting legitimate map-key Evidence into the map-value carrier;
- forging a tuple shape witness;
- clearing semantic error and forging an otherwise well-shaped positive `binary-operator-types` Evidence for rejected `true + false`.

All modified candidates must be denied. `test_quad_native.py` compares eight native-vs-frozen-C0 positive/negative programs, including maps, tuples, if expressions, iteration and invalid typing, and separately checks the real Q3.9 fixture.

## Explicit trust limits

**Not an opaque constructor:** `Evidence`, `Sem` and `TrustedSemAdmission` remain ordinary public Semantic records. Any caller can syntactically manufacture them. The defended surface is the production **`compile_source(text)` pipeline**, where `q3_trusted_admit` is invoked and only canonical replay state is lowered.

**Not an independent source authenticity proof:** replay starts from `candidate.prog`, a parsed AST in memory, not a separately authenticated source-file hash or stable multi-file source identity. A party capable of changing the trusted Sema implementation, parsing logic, AST itself or driver can still change the result. The boundary is deterministic local recomputation, not a cryptographic seal.

**Scope of comparison:** maps are compared for keys in the AST node-ID domain. Extra out-of-domain entries are not enumerated; they cannot participate in lowering because the driver uses replayed canonical state. Internal string-keyed name indexes are regenerated from the AST and the driver never forwards candidate indexes. This is an admission *consistency* check followed by canonical transfer, not an exhaustive equality proof of every `Sem` field.

**Cost:** one extra full Sema analysis per accepted compiler invocation plus per-node comparison. This may increase Steps/Calls and should be profiled under an identical workload before optimizing. Full scope qualification and exact SHA are in the checkpoint.

**Out of scope:** language-level opaque ADT/visibility, authenticated module implementation, capability-based constructors, multi-file source provenance, proof transport through SemCode, frozen C0/VM/Verifier API changes, C1/C2 fixed point and Bootstrap Seal. No authority-bearing proof format is claimed.
