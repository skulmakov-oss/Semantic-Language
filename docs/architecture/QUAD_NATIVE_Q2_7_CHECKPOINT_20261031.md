# Q2.7 / Q3.6 / Q4.6 — Tuple Destructuring and Inferred-Type Origin Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`.
**Disposition:** local experimental candidate, uncommitted and unpublished; no normative self-hosting seal.

## Implemented

- **Q2.7:** a separate `tuple-destructure-shape` carrier proves tuple kind/arity for both `k=40` and `k=41`; `tuple-annotation-type` proves explicit tuple type compatibility for annotated `k=40`; `tuple-assignment-items` checks ordered existing-target compatibility for `k=41`. A negative shape produces `F`, and a mixed compatible/incompatible target set produces `S` within the one assignment proposition, without joining unrelated facts.
- **Inference:** `Sem.tuple_inferred_components` records the inferred `TyInfo.kids[i]` forwarded to each new bound name; no artificial positive Evidence is generated for unannotated inferred bindings.
- **Q3.6:** all tuple binding/assignment statements require exact, valid, T-only `tuple-destructure-shape` with the RHS position. Annotated bindings require an additional `tuple-annotation-type` proof; nonempty tuple assignments require the target-name-position `tuple-assignment-items` proof. Inferred binding origin metadata must exist for every target and equal the corresponding RHS inferred tuple component type, with a destination slot. Missing/forged carriers or missing/mismatched component origins fail closed before lowering.
- **Q4.6:** internal `FnIr.proofs` carries validated shape and compatibility evidence. **Separate** `FnIr.tuple_inferences: Sequence(TupleInferenceLink)` preserves origin, target binding, component index, resolved type and two source-local byte positions. These links are not evidence and cannot create or join T/F/S observations.

## Test evidence

- `tests/compiler/programs/quad_tuple_inference_internal.sm` executes real Sema→Lower and inspects shape/assignment evidence, unannotated type origins, IR proof records, independent inference links, tampered and missing proofs, wrong inferred component types and deterministic replay.
- `tests/compiler/test_quad_native.py` adds 10 existing tuple case comparisons with pinned frozen C0 for successful and rejected programs. `r34_tuple_expression_call` intentionally executes `assert(false)`, so runtime failure is the expected behavior for both compiled outputs.
- Targeted Q2.7 tests: **2/2 PASS** after correcting the r34 runtime expectation. Frozen C0 compiled the real internal test; frozen Verifier and VM passed under explicit bounded research budgets.
- Full local suite and full differential suite qualification follow below when completed.

## Trusted identity (local experimental, not canonical source set)

- `compiler/sema.sm` SHA-256 `d5433f909e9c6c276b30ea934fce4b0a0f602dc99d11d6b7a183c2090b5cc6a1`.
- `compiler/lower.sm` SHA-256 `f0b5bf49ce6bc79ac4475fc67c1e27710fe65d4979c07e09815123d689abefed`.
- `tests/compiler/test_quad_native.py` SHA-256 `802cef067b42893f5d5c35dd773ed0ad13da970e463db904a7d73aebf5c5b668`.
- `tests/compiler/programs/quad_tuple_inference_internal.sm` SHA-256 `5b550388be6c592f9c61b1c20eea30f42f012532d093e77955cd647b540146b3`.
- Experimental source-bundle SHA-256 `b1f04eeabd229e5ad7b8aea2391876905c781feffb6d3112bd741d5571ca9c65`.
- Frozen C0 executable, Verifier, VM, Harness, SemCode format and Bootstrap Contract were not modified.

## Limits

This does not close every type relation in Q2: condition types for `if/while/assert`, `for-in` admissibility, general unannotated `let` inference, exhaustive operator/context rules, ownership/effect proof obligations and verified source origin authority remain open. File-qualified spans, signed/opaque derivation carriers, lineage through optimization and verifier-facing evidence serialization are still outside Q4.6. Raw Evidence is forgeable, and proof checks compare metadata to the same in-memory AST rather than independently authenticating it. C1/C2 canonical fixed point has **not** been established.

**No commit, push, PR, merge or protected-code edit was performed.**

## Final local qualification — same candidate after Q2.7 patch

- Pinned frozen C0 compiled the candidate and frozen verifier accepted **67 functions** (`SEMCOD23`, epoch `0.24`).
- Full `python -B -m unittest test_quad_native test_imports -v`: **24/24 PASS**.
- All ten targeted legacy tuple cases matched frozen C0 admission; accepted cases verified; intentionally failing `r34` preserved its runtime assertion error.
- Full compiler differential: **123/124 PASS**, nine SemCode mutations **9/9 PASS**. The **only** mismatch is pre-existing `n24_unsupported_type_annotation`, whose legacy expected failure conflicts with quad-native admission. The differential process exited **nonzero**. It must **not** be reported as all green.
- Differential results: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031-q27\differential\results.json`.
- Golden quad differential artifact unchanged: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`.
- Latest golden VM metrics **679,352 Steps / 8,841 Calls / 12 peak Frames / 3 effect Calls**, compared with Q2.6 **656,533 Steps / 8,527 Calls**. This is a workload observation, **not** a controlled optimization benchmark.
- Harness and `git diff --check`: PASS before final doc append, then re-run below. C1/C2 fixed point: **NOT RUN**.
- **Disposition:** Q2.7 / Q3.6 / Q4.6 experimental checkpoint accepted. Q2 complete coverage, Q3 universal verified admission, Q4 verifier-visible provenance, canonical source-set qualification, and Bootstrap Seal remain **open**. No commit, push, PR or merge, frozen compiler/VM/Verifier/Harness/Bootstrap Contract edit.
