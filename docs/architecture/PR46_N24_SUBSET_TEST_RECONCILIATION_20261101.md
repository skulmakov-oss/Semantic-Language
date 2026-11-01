# PR #46 — n24 test-corpus reconciliation (2026-11-01)

## Decision and scope

Repository-owner request: correct the known `n24_unsupported_type_annotation` mismatch after the PR #46 P2 fixes. This checkpoint changes **test-case selection**, not Semantic language semantics or the normative Bootstrap Contract v2. No production compiler, verifier, VM, C0 reference, Harness, source-set manifest, or bootstrap policy is changed.

Prior corpus state (124 cases):

- `n24_unsupported_type_annotation` contains `fn f(q: quad)`.
- The subset expected `reject:unsupported:type 'quad'`, whereas the pinned C0 reference expected `ok` and the candidate accepted it.
- Therefore the full differential yielded **123/124**, exit 1; this was a genuine expectation contradiction, not a compiler failure.

## Resolution

1. Preserve `n24_unsupported_type_annotation` as a **negative subset-boundary case**, replacing `quad` with `f64`. Expected result: `reject:unsupported:type 'f64'`. The current compiler explicitly rejects `f64` as written, and pinned C0 accepts the same program. The pre-existing `reference_cases.json` value `n24 = ok` remains unchanged.
2. Preserve the original `quad` source as a separate **positive case**, `p24_quad_type_annotation`, expected `ok`. It must compile deterministically to SemCode, pass verifier admission, and execute on both pinned C0 and dev roles.
3. Regenerate corpus deterministically from `tests/compiler/make_cases.py`, retaining the untouched 123 previous case bodies and expectations.
4. Keep the frozen Bootstrap Contract v2 and `docs/BOOTSTRAP_SUBSET.md` registry unchanged. This correction **does not admit `quad` to the bootstrap-required source subset**, does not create a canonical `S`, and does not establish C1/C2 fixed point or Bootstrap Seal.

## Direct source-behavior probe (before corpus edit)

- `fn f(q: f64)`: pinned C0 `smc check` PASS; candidate prints `error unsupported ... type 'f64' (as written) is outside the supported subset`.
- `fn f(q: quad)`: pinned C0 `smc check` PASS; candidate emits `hex ...` SemCode.

## Qualification

Run full identity-checked `tests/compiler/run_tests.py` using the attested C0 checkout, binary, provenance and anchor, with `SEMANTIC_TEST_OUT` outside the source tree. Confirm all existing positives, negatives, O0/O1 reference parity, deterministic compilation, verifier/VM execution, and all nine mutations. Also run `test_corpus_generation.py`, compiler unit suites, `bootstrap/validate_contract.py`, local Harness, and `git diff --check`.

Expected corpus size after reconciliation: **125** (124 original cases, one additional positive case). A successful exit is a **compiler regression-suite result only**; it is not a Bootstrap Seal or an independent C0-vs-dev implementation comparison if both roles use one attested C0 executable.

## Verified results

- Full pinned-C0 identity-checked differential: **125/125 cases PASS**, **9/9 mutation checks PASS**, `QUALIFICATION_EXIT=0`.
- Corrected negative `n24_unsupported_type_annotation`: PASS; independent positive `p24_quad_type_annotation`: PASS. Reference C0 and dev roles use the same attested C0 executable, not independent implementations.
- Bootstrap Contract v2 validator: PASS; source-set identity unchanged (`sha256:ad9f28cfa96a5e2afd02aa9ce1c386b472ec8f8f4cead7fdb87acd2188ba7cd3`). Harness: PASS; staged `git diff --check`: PASS.
- Extended `unittest discover` (52 tests) exited **FAIL** due to three subtests of `test_invalid_ownership_anchors_fail_before_artifact` (missing/wrong_store/wrong_record), each exceeding the fixed 50,000-Call envelope. Those tests are unrelated to the n24 corpus edit and have **not** been waived or reclassified. Their quota limits must not be raised to create a green result.
- Do not represent these compiler regression results as a complete repository qualification or Bootstrap Seal. The driver Q4.9 source/AST and Q3.9 canonical-Sema replays remain untouched pending a separate architectural decision.

Evidence path: `%TEMP%\pr46-20261101-n24-reconciliation\results.json`. Initial qualification used compiler HEAD `9a9e818c433ce46b29201afb668ed56235fb2d5f` without modifying compiler sources; a separately scoped subsequent driver single-pass optimization has its own report (`PR46_DRIVER_SINGLE_PASS_20261101.md`) and exact final-source requalification.
