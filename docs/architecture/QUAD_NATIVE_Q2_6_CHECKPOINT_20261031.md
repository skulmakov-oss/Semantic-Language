# Q2.6 / Q3.5 / Q4.5 — Map<K,V> Dependent Evidence Checkpoint

**Date:** 2026-10-31. **Branch:** `feat/quad-native-shf15`. **Base HEAD:** `972066b25d2201344747becff0a0d6abfeb0bf4d`. **Disposition:** uncommitted, unpublished experimental vertical slice.

## Implemented

- `compiler/sema.sm` records independent `map-context-type` / `map-container-type`, `map-key-type`, and `map-value-type` evidence for `map_empty`, `map_contains`, `map_get`, `map_set`.
- The three proof kinds never JOIN together. Each observed compatible type produces one `T` witness; incompatible container, key or value produces its own `F`. Missing required Map context produces `N` for `map-context-type` and the existing source diagnostic. Type errors remain fail-closed.
- `compiler/lower.sm` detects map builtin obligations from reserved AST names and checks the expected callee code independently; deleting a `callees` entry cannot erase the obligation. It verifies each required proof independently, with exact identity, `T` state and AST source offset before lowering.
- `FnIr.proofs` preserves non-emitted `PhaseProof` snapshots for container/context, key, and value in deterministic order. Context-only `map_empty` uses the builtin call AST position as an observation marker, not a fabricated argument witness.
- Normative SemCode, Verifier, VM, frozen C0 and Bootstrap Contract remain unchanged.

## Scope of tests

- `tests/compiler/programs/quad_map_dependency_internal.sm` exercises the real parser, Sema, Q3 and lowered IR. It checks all four builtins; state `N/F/T`; missing map context; incompatible container/key/default/value; source offsets; provenance order; deterministic replay.
- Negative tamper tests remove **only** map key/value/callee proof, corrupt exact value-source position, substitute valid key evidence into the value slot, and try a cross-proposition JOIN. All must fail closed while unrelated valid observations remain intact.
- `tests/compiler/test_quad_native.py` adds direct native vs frozen C0 tests of the twelve existing `m01..m12` map scenarios, verifying and executing accepted compiled artifacts in both implementations.
- Scope does not extend to general dependent type inference, map aliasing across generics, file-qualified origin chains, C0 verifier-readable proof serialization, ownership/effect lowering preconditions, or a normative self-hosting seal.

## Qualification (to be appended after final checks)

Pinned C0 compiler built the new candidate and frozen Verifier accepted 64 functions, `SEMCOD23`, epoch `0.24`. Targeted Q2.6 fixture and the map legacy comparison passed separately. Full unit/import, differential mutation and Harness qualification results are recorded below once verified.

**Trust limitation:** constructible `Evidence`, in-memory `Sem` records and source-local `Node.pos` checks do not make type propositions cryptographically authenticated or unforgeable. C1/C2 fixed point remains not attempted. No commit, push, PR or merge.

## Final verification results — 2026-10-31

| Gate | Result |
|---|---|
| Frozen C0 compile of experimental compiler | **PASS** |
| Frozen C0 verify candidate | **PASS**, 64 functions, `SEMCOD23` epoch `0.24` |
| Focused Q2.6 tests | **2/2 PASS** including 12 C0 map case comparisons |
| Full quad/import unit suite | **22/22 PASS** |
| Full compiler differential | **123/124 PASS**, only previous `n24_unsupported_type_annotation` mismatch |
| SemCode mutation tests | **9/9 PASS** |
| Harness, diff check | PASS (revalidate after append) |
| C1/C2 canonical fixed point | **NOT RUN** |

Full differential artifact: `C:\Users\said3\AppData\Local\Temp\chatgpt-shf15-20261031-q26\differential\results.json`; full differential process exits nonzero because `n24` remains unresolved. It must **not** be reported as complete green qualification.

Verified source and fixture identities:
- `compiler/sema.sm` SHA-256: `bd9377c3de064a3c462d31c4574d3b0ee3f5649ac75bde75eaf16a678830d933`.
- `compiler/lower.sm` SHA-256: `5b3002c2c74516594ee72649a947a09a4a26afb8485ccc38c403515070034a74`.
- `tests/compiler/test_quad_native.py` SHA-256: `de353c8b07bb1ee701d54f253e9327f3f7e9f8386f1e23411888645a78da3867`.
- `tests/compiler/programs/quad_map_dependency_internal.sm` SHA-256: `18a4f3f9861d2d46ce4381d0a09d87ca2ba991d399f461677486648db8c06645`.
- Experimental source-bundle identity: `c61daf45f22d58296c7f4b6b39d100607ca4d348da35d328819071686be078c3` (a local nine-module content hash, **not** normative canonical source-set identity).
- Golden quad differential artifact remains `178aab9528d40609350e1db73ca8952e0d482118972f18db36c2bef32d3b77ba`.
- Same golden test VM metrics: 656,533 Steps, 8,527 Calls, peak 12 Frames, 3 Effect Calls (Q2.5 observation: 636,980 Steps / 7,609 Calls). Different source revisions mean this is **not** a controlled optimization benchmark.

**Disposition:** Q2.6 / Q3.5 / Q4.5 accepted locally as an experimental vertical slice; full Q2/Q3/Q4 closure and Bootstrap Seal are not claimed. No commit, push, PR or merge; frozen C0, verifier, VM, Harness scripts and Bootstrap Contract unchanged.

## Subsequent engineering boundaries

1. Enumerate remaining semantically significant constraints not covered by Q2, especially tuple pattern/assignment, inference relationships, and ownership/effect assumptions.
2. Extend Q3 to check those obligations and strengthen authority beyond forgeable source records and mutable evidence maps.
3. Extend Q4 to file-qualified source spans, type-derivation parent links and a separately authorized versioned proof transport contract for Verifier.
4. Reconcile legacy `n24` only through explicit owner governance; separately qualify canonical C1/C2 fixed-point at required quotas.
