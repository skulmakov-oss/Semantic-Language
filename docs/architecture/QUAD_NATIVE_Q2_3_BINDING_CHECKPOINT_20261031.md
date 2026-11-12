# Q2.3 / Q3.2 / Q4.2 — Binding, Assignment and Return Evidence Checkpoint

**Date:** 2026-10-31
**Branch:** `feat/quad-native-shf15`
**Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`
**Disposition:** experimental local checkpoint; no commit, push, merge, Bootstrap seal or contract change.

## Implementation

- `compiler/sema.sm` extends `Sem.type_evidence` to `annotated-binding`, `assignment-type`, `return-value-type`.
- Each has a single, independent compatibility test comparing actual and required types. State **T** = matching witness; **F** = mismatched witness. Do **not** fabricate S from an unrelated fact or a single comparison.
- An inferred declaration has no independent expected type and does not generate a synthetic proof. A unit `return;` without a value similarly does not represent a return-value comparison.
- The original `type` diagnostic category and existing messages are preserved. A rejected program may retain negative `Evidence` in Sema, but is never admitted to lowering.
- `compiler/lower.sm` extends `q3_required_kind` and `q3_source_nodes` for AST `k=10` explicitly annotated bindings, `k=11` assignments, and `k=14` explicit value returns. Missing, invalid, non-T or wrong-position proof is rejected before artifact emission.
- `lower_stmt` now records `PhaseProof` for admitted statement obligations in `FnIr.proofs`, adding statement-level provenance to previous expression-level Q4 transport. SemCode encoding is unchanged.

## Test evidence

- `tests/compiler/programs/quad_binding_return_internal.sm`: real C0-compiled Semantic analyzer and lowering, direct T/F witness checks, return/assignment/binding evidence, correct positional origins, missing-proof rejection, forged but structurally valid wrong-position witness rejection, and inspected `FnIr.proofs`.
- `tests/compiler/test_quad_native.py`: native candidate versus pinned C0 compile/admission/execution comparison for seven valid/invalid source examples (annotated declarations, inference, assignments and returns). The pre-existing Q4 fixture was expanded to expect three additional annotation proofs rather than silently losing evidence.
- The internal fixture's execution mode is the pinned C0 `run` with explicit research budgets **1,000,000 Steps / 50,000 Calls / 256 Frames** because `run-smc` uses a fixed 100,000-step limit. This is a **test budget, not a changed normative Bootstrap quota**.
- Full compiler differential: **123/124** test cases pass; **9/9** mutation tests pass. Remaining mismatch: legacy `n24_unsupported_type_annotation` expects `quad` rejection despite the quad-native candidate admitting it. Overall differential process exits nonzero.
- Differential evidence: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031\q2_binding_full_diff\results.json`.
- Final full unit/import suite and Harness status: see execution log / subsequent checkpoint review before claiming a canonical green qualification.

## Authority and exclusions

Neither the frozen C0 binary, VM, Verifier, Harness code nor Bootstrap Contract was changed. No canonical source-set registration, C1/C2 fixed point or normative-budget qualification was performed. `Evidence` remains a constructible source record; `evidence_valid` checks shape and consistency, and Q3 checks AST position consistency, but these checks **do not authenticate type claims** against a maliciously forged positive carrier having a correct source offset. The provenance currently has no file-qualified spans or derivation ancestry.

## Remaining work

1. Q2: tuple/sequence elements, matcher arm result typing, builtin argument contracts, operators and type inference relations. Define exact proposition identity and evidence flow for each.
2. Q3: cover additional lowering assumptions, origin authority and ownership/effect constraints; do not imply that uncovered AST forms are proved.
3. Q4: file-aware source spans, checked transformation/derivation lineage, trustworthy carrier construction, serialization and verifier-facing evidence under separately authorized contract governance.
4. Qualification: reconcile n24 through owner governance; then independently verify canonical source set, pinned C0, full ordinary/differential suite and C1/C2 fixed point.

## Final local qualification (completed)

- `python -B -m unittest test_quad_native test_imports -v`: **16/16 PASS** on the final fixture set.
- `git diff --check`: PASS.
- `scripts/harness-check.ps1`: PASS, **25 changed paths** within the authorized scope.
- Frozen C0 compilation and verification of `q2_binding_candidate.smc`: PASS, **55 functions**, header `SEMCOD23`, epoch `0.24`.
- Full differential: **123/124**, all **9 mutation tests PASS**, nonzero process exit due to the previously known `n24_unsupported_type_annotation` only.
- `compiler/sema.sm` SHA-256 `c6701b1dd0e4a6448cf329ccd8de008de78b5adcafa8d16d5e2374ee4db86f6b`.
- `compiler/lower.sm` SHA-256 `79fb33b4bde63b58c40f58ceab94428f94172eef8bc80c4ccd0b6a6e0417cac8`.
- `tests/compiler/test_quad_native.py` SHA-256 `f0bb64969382a05c3b694a9b1c4a47ebe91bb2852b9e4b436da5963b98bf70aa`.
- Golden quad differential artifact SHA-256 remains `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`; current test VM metrics for that scenario: **520,521 Steps / 5,261 Calls / 12 Peak Frames / 3 Effect Calls**.

**No publication, commit, PR or merge occurred.** This checkpoint is not a statement of full Q2–Q4 implementation or canonical self-hosting qualification.
