# Q2.4 / Q3.3 / Q4.3 — Container and Match Evidence Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`.

**Disposition:** uncommitted and unpublished experimental candidate. No frozen C0, VM, Verifier, Harness or Bootstrap Contract changes; no canonical fixed-point claim.

## Implemented vertical slices

Q2.4 adds three independently scoped proposition families to the existing six: `tuple-target-items` for tuple literals with an actual expected tuple type, `sequence-elements` for nonempty Sequence literals, and `match-arm-types` for value-producing match arm results. All use checked witness merging; conflicting evidence is preserved as `S`, single disconfirmation as `F`, all confirming evidence as `T`. The distinction between contextual type obligations and type inference is retained.

Q3.3 extends fail-closed pre-lowering validation to those covered AST forms: exact carrier identity, valid state, T only, number of contributors, AST-node source position for each witness. Required match proof detection uses the separate `Sem.match_result_checked` registry, set before match-arm checking. Deleting match Evidence alone is not permitted to bypass Q3.

Q4.3 carries these checked source-node IDs, offsets and Evidence into internal `FnIr.proofs`; the frozen serialized SemCode and verifier protocol are not altered. The actual test `quad_container_match_internal.sm` directly inspects the real Sema and Lower implementations, not a substitute model.

## Targeted test evidence

- 9 good/bad native-vs-C0 programs cover contextual tuple conformance, inference without artificial proof, homogeneous/heterogeneous and empty sequences, and matching/mismatching `match` arms.
- Repeated programmatic introspection checks T/F/S and both witness families, source position mapping, and absence of synthetic proof when the source has no element observations.
- Negative proof tests cover structurally valid but false source positions, an empty evidence map, and **removal of only match Evidence** while preserving the independent match obligation registry and other proof carriers.
- Actual compiler is built and verified by pinned frozen C0; tests run under separately bounded research budgets. Full qualification results and final hashes are appended after checks complete.

## Limitations and next checkpoint

- `Sem.tuple_expected` and `Sem.match_result_checked` remain mutable RAM maps; they establish obligations inside the experimental Sema data flow, **not** an unforgeable type authority.
- Source locations are compilation-unit local byte offsets, not file-qualified source spans or derivation ancestry.
- Neither the return-path evidence origins, all builtin signatures, numeric operators, all tuple destructuring constraints, nor ownership/effect preconditions are covered by Q2–Q4.
- Non-emitted `FnIr.proofs` do not give a frozen C0 verifier evidence to check.
- The old `n24_unsupported_type_annotation` expected negative result conflicts with quad-native admission and is subject to a separate owner-controlled contract decision.
- C1/C2 fixed point and normative source-set/budget qualification remain **not run**.

**No git commit, push, PR or merge was performed.**

## Final local qualification — post match-registration safety patch

- **Frozen C0 build:** PASS; compiler candidate `final_candidate.smc` generated, 252,460 bytes.
- **Frozen C0 Verifier:** PASS, 55 functions, `SEMCOD23`, epoch `0.24`.
- **C0 pinned executable SHA-256:** `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e` (unchanged).
- **Unit and import regressions:** `python -B -m unittest test_quad_native test_imports -v` **18/18 PASS**, including missing-match-only Q3 rejection.
- **Full compiler differential corpus:** **123/124 PASS**; process exit nonzero due to previously known `n24_unsupported_type_annotation`. **9/9 mutations PASS.** This is **not** a full green qualification.
- **Full differential evidence:** `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031-q24\final_differential\results.json`.
- **Harness and diff check:** PASS in qualification (scope 28 changed paths before this checkpoint appendix; rerun after documentation changes).
- **Golden quad differential artifact SHA-256:** `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`; latest test metrics **550,767 Steps / 5,428 Calls / 12 Peak Frames / 3 Effect Calls**.
- **Experimental source bundle SHA-256:** `9c124902aa8b792c6bdc875727828b3af7ef0bea3829258f83e0c7f96a254697` (experimental nine-module content framing, **not** canonical Bootstrap source-set identity).
- **compiler/sema.sm SHA-256:** `4d12437e4a7f397226ee29f08228c7fcb1b64f84b458b7644e9aec6ed894d870`.
- **compiler/lower.sm SHA-256:** `f73fd5c669d004165fe02e72f7ef2ae96a52894105b209b1993a9f0aaf9d00bf`.
- **tests/compiler/test_quad_native.py SHA-256:** `b78f9ef22f3674e61d3dd2d678cb8a017a4e6e0ca17df505a01d12fd23c8b1c6`.
- **quad_container_match_internal.sm SHA-256:** `5bb2541cd72253cc0f7d9c890e0e105f1f498d022b9cffa160ec981e5aadf05b`.

**Remaining:** Q2.5 builtin/operator type evidence, complete symbol/type inference constraints, provenance authority; Q3 universal IR/ownership/effect preconditions; Q4 file-qualified provenance + governed verifier-facing transport; n24 governance; canonical C1/C2 fixed-point proof.

**No commit, push, PR, merge or frozen contract modification.**
