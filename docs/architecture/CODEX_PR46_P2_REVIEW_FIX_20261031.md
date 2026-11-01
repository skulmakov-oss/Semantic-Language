# PR #46 — Codex P2 review follow-up (2026-10-31)

**Scope:** fix three independently reported Codex code-review findings in existing Draft PR #46. **Base:** `feat/shf-self-coverage`. **Head:** `feat/quad-native-shf15`. Do not merge or modify frozen C0, Verifier, VM, Bootstrap Contract or Harness.

## 1. Parser: EOF lookahead

**Codex thread:** https://github.com/skulmakov-oss/Semantic-Language/pull/46#discussion_r4430684043

**Defect:** Fast-path initializer, assignment and binary-RHS code read `p.toks[p.i + 1]` unconditionally. If `p.i` refers to the unique EOF token, the peek is out of bounds and traps the compiler VM rather than producing a syntax diagnostic.

**Repair:** `compiler/parser.sm` centralizes guarded lookahead in `safe_lookahead_kind` and `safe_lookahead_token`, using the existing EOF token as the fallback. No unchecked `p.toks[p.i+1].kind` or `p.toks[p.i+2].kind` remains in the parser. `parse_expr` retains its normal `expected expression` diagnostic path.

**Regression:** eight truncated inputs in `test_codex_review_parser_eof_lookahead`, including `let x =`, annotated `let`, assignment, `1 +`, `true &&`, `assert(1 +`, and a truncated sequence. Assert syntax rejection with an ordinary compiler outcome and no VM trap.

## 2. Wire-domain u16 operands beyond lookup cache

**Codex thread:** https://github.com/skulmakov-oss/Semantic-Language/pull/46#discussion_r4430684047

**Defect:** The cached `u16` representation has 3,328 entries, but many sites indexed it directly with register IDs, ignoring the valid unsigned 16-bit values outside the cache.

**Repair:** `compiler/semcode.sm` introduces `encode_u16_le` (cached low-value path, arithmetic little-endian fallback) and replaces all former direct register, argument, count and ownership-related lookup indexing sites. The wire format and existing cached output remain identical for values in the cached domain.

**Regression:** actual Semantic encoder fixture tests boundary 3327 → `ff0c`, 3328 → `000d` and 65535 → `ffff`, and serializes a synthetic function using registers 3328 and 65535. This is an actual encoder test, not a Python mock.

## 3. Full 32-bit jump, ownership anchor and function length

**Codex thread:** https://github.com/skulmakov-oss/Semantic-Language/pull/46#discussion_r4430684052

**Defect:** Jump-offset branches wrote only low 16 bits plus `0000`, and OWN0 anchor PCs also discarded upper bytes; large offsets could additionally index past the 256-entry byte table.

**Repair:** `encode_u32_le` serializes both little-endian u16 halves. Both jump opcodes, OWN0 anchor PCs and the 32-bit function-length field use full-width encoding. Existing small offsets produce identical bytes.

**Regression:** the real Semantic encoder fixture verifies 65535 and 65536, then creates a 7,500-instruction synthetic function with code offset 67510 and ownership anchor 67505, checking both four-byte addresses and the function length without truncation. Encoded byte sequences are asserted exactly.

## Test and governance notes

- New fixture: `tests/compiler/programs/quad_codex_review_internal.sm`.
- New Python tests: `test_codex_review_parser_eof_lookahead` and `test_codex_review_semcode_16_32bit_boundaries`.
- Frozen C0 is the execution oracle and verifies the built internal fixture.
- Full quad-native/import suite, canonical differential, mutation tests and Harness must be requalified on the final commit. Record actual results before resolving review threads.
- Previously known `n24_unsupported_type_annotation` is a separate unresolved legacy-vs-quad contract mismatch; no silent test expectation changes.
- The full semantic typing, provenance and ADT trust limits of Q2–Q4.9 are unchanged.
- PR remains Draft and stacked on PR #45; resolution of these P2 threads is **not** an approval to merge or a normative C1/C2 Bootstrap Seal.

## Final local qualification, post-fix

- Frozen C0 compiled the modified compiler and Verifier accepted **92 functions** (`SEMCOD23`, epoch `0.24`).
- The full Python quad-native/import suite: **32/32 PASS**, including two newly added P2 regression tests.
- Full compiler differential: **123/124 PASS**; the **only** failing case remains the known pre-existing `n24_unsupported_type_annotation` legacy-vs-quad contract mismatch. The runner exits **RC=1**; this is **not** a complete green qualification.
- SemCode mutation suite: **9/9 PASS**.
- Golden quad differential artifact SHA-256 unchanged: `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`.
- Golden VM profile on this candidate: **1,601,934 Steps, 28,636 Calls, 13 peak Frames, 4 Effect Calls**. Previously Q4.9: 1,578,534 Steps / 27,346 Calls. The modest increase should not be mistaken for a controlled benchmark.
- Full differential evidence: `C:\Users\said3\AppData\Local\Temp\quad_review46_differential\results.json` (machine-local, **not committed**).
- Protected code, frozen C0, VM/Verifier, Harness scripts and Bootstrap Contract are unchanged.

**Disposition:** all three Codex P2 findings fixed and regression-qualified in the PR branch. Review threads may be resolved after the commit has been pushed and the remote SHA independently confirmed. Keep Draft; request Codex to re-review the new commit.
