# Q2–Q4 Experimental Checkpoint — 2026-10-31

**Worktree:** `C:\Users\said3\Desktop\Semantic-Language-worktrees\quad-native-shf15`
**Branch:** `feat/quad-native-shf15`
**Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`
**Status:** local uncommitted / unpublished experiment; not a canonical Bootstrap Seal.

## Implemented

| Milestone | Implemented evidence | Open scope |
|---|---|---|
| Q2 — Full type constraints | `if-branch-type`, `call-arguments` for named functions, `record-fields` for literals/with: witness-preserving quad aggregation | Other type contexts, builtins, return/assignment constraints, tuple/sequence, match, inference, symbol resolution |
| Q3 — Lowering preconditions | Fail-closed `q3_lowering_error` at `compile_source` boundary; exact owner/identity/state/count/AST position validation for covered Q2 obligations | General IR/ownership/effect preconditions; some AST operator families not yet covered |
| Q4 — Cross-phase provenance | Real `Sema Evidence → Lower PhaseProof → FnIr.proofs` transport, retaining source-node IDs, byte positions, polarity-bearing Evidence, deterministic order | File-aware spans, proof lineage across optimizations, encoded proof ABI, verifier-visible checks |

## Qualification from the exact local candidate

- Pinned frozen C0 builds the compiler; frozen verifier accepts **55 functions**, header `SEMCOD23`, epoch `0.24`.
- `python -B -m unittest test_quad_native test_imports -v`: **14/14 PASS**.
- Full compiler differential: **123/124 cases PASS, 9/9 mutation checks PASS**; process exit nonzero because legacy `n24_unsupported_type_annotation` rejects a `quad` type now admitted by the candidate and pinned C0.
- Harness scope: PASS; `git diff --check`: PASS.
- Full differential evidence: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031\q2q3q4-full-differential\results.json`.

**Important:** `123/124` is not full qualification. The `n24` contract mismatch requires owner-level resolution. No C1/C2 fixed point was attempted; the normative quota / canonical source set / frozen verifier gate were not changed.

## Next acceptance gates

1. **Q2-COVERAGE** — enumerate all remaining typed AST edges and their explicit proposition identities, then add negative/inference/diagnostic fixtures without changing existing error categories.
2. **Q3-ENFORCEMENT** — extend preflight to every lowered AST family that consumes semantic assumptions, including builtin contracts, assignment/return obligations, ownership and effect policies; distinguish source failures from internal provenance failures.
3. **Q4-PROVENANCE** — introduce stable `(source file, span, source node, derivation parent)` identities and a separately authorized transport/serialization contract; qualify provenance against deterministic replay and verifier authority.
4. **FULL QUALIFICATION** — reconcile the legacy n24 rule through governance, run full workspace/differential and then independently demonstrate canonical C1/C2 fixed point at the applicable normative quotas.

## Non-actions

No git commit, push, PR, merge, frozen C0 change, VM/verifier edit, or Bootstrap Contract modification was authorized or performed in this checkpoint.
