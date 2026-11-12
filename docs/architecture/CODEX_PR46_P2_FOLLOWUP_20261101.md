# PR #46 — Codex P2 follow-up: interleaved imports and evidence scaling

**Date:** 2026-11-01. **Starting PR head:** `b9396c7260b5ea5bb1e027d985beee73f0747be4`.
**Review threads:** [import scanning](https://github.com/skulmakov-oss/Semantic-Language/pull/46#discussion_r4436380227) and [witness accumulation](https://github.com/skulmakov-oss/Semantic-Language/pull/46#discussion_r4436380229).

## Corrections

1. `compiler/main.sm` now continues scanning every source line after declarations, not only the initial contiguous import block. It retains original-source segments between import directives so file-qualified Q4.9 source spans remain consistent. Covers imports after declarations, multiple separated imports, and nested CRLF cases.
2. `compiler/sema.sm` uses a per-analysis witness membership index for `accumulate_type_constraint`: each new witness is checked by owner, source position and polarity instead of repeatedly validating all preceding pairs. The externally callable `evidence_valid` remains fail-closed and its duplicate check now uses linear membership rather than pairwise comparison. Positive and negative witnesses at one source offset remain separate polarities; duplicates within either polarity are rejected.

## Qualification on the fixed local candidate

- Pinned frozen C0 compilation: **PASS**, verifier admits **92 functions** (`SEMCOD23`, epoch `0.24`).
- Added regression tests: **3/3 PASS** (`tests/compiler/test_pr46_followup.py` and its real Semantic fixture).
- Existing quad-native tests: **27/27 PASS**; import tests: **5/5 PASS**. Total targeted/unit: **35/35 PASS**.
- Stress case, `Sequence(i32)` literal with 1,000 elements, under `trusted-compiler` (`20,000,000 Steps`, `50,000 Calls`): **PASS**; **3,866,718 Steps / 40,762 Calls**, 12 peak frames, 4 effects. Before repair this workload exceeded the 20M-step quota.
- Full pinned-C0 identity-checked compiler differential (both roles use the same attested binary): **123/124 PASS**. The *sole* failure remains the pre-existing `n24_unsupported_type_annotation` legacy-vs-quad contract mismatch. The runner returned **RC=1**: this is **not** a green qualification.
- SemCode mutation checks: **9/9 PASS**.
- Golden quad differential SHA-256 unchanged: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`.
- `git diff --check` and Harness path-scope check: **PASS**.
- Full differential evidence (machine-local, not committed): `$TEMP/pr46-20261101-reviewfix-differential/results.json`.

## Boundaries and disposition

Only the experimental compiler, tests and this review document are changed. Frozen C0, VM/verifier implementation, trusted Harness, Bootstrap Contract, source manifest and licensing are untouched. Q3.9 canonical replay continues to be the trusted production admission path; this is not an opaque authenticated Evidence ADT. No C1/C2 fixed-point proof, normative bootstrap seal, PR merge or release is claimed.

**Disposition:** two P2 findings repaired and regression-tested; safe for another PR review, **not** a merge approval. The `n24` exception still requires explicit separate governance.
