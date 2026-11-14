# Quad-native compiler: current-main compatibility checkpoint

## Baseline and authority
- Main base: `242e51fabc4cf4507428653fc3596c0074e49ba1` (PR #62).
- Quad-native source: PR #46 `bd2206fae01d3a3052eee3359c23c28551a93e0f`.
- Frozen C0 SHA-256: `a612fc710c0961103aff6f0d078f442214815d8c6a49db2ea163c6bddf3b758e`.
- Do not modify C0, `.harness/**`, `bootstrap/**`, `reference/**` or trusted checkers.

## Integration scope
The original 3-file merge cannot be resolved by choosing either parent: the PR side loses `lower_args` linkage to the merged lowering module; the main side loses the Quad-native `Evidence` type. The coherent candidate starts from all nine Quad-native compiler modules and restores recent main admissions without accepting stale governance changes.

## Compatibility repairs
1. Admit contextual field identifier `text` in record initialization, declaration and access.
2. Parse and type-check `Option::Some(payload)` / `Option::None`, preserving a rejection for context-free `None`.
3. Admit scalar/Quad and acyclic structural equality while excluding unsupported recursive and map carriers.
4. Extract Option type-checking into a helper to stay within the Frozen C0 4096-register function budget.
5. Encode Option construction using SemCode opcode 71 and remap OWN0 FieldSymbol indices to global runtime symbols.
6. Move legacy n24 rejection to `f64`, add p24 `quad` admission, and keep source generator in sync.

## Collected evidence
- Frozen C0 compilation and verifier: PASS, 94 functions, SEMCOD23 epoch 0.24.
- Four self-host grammar differential cases from #61: three byte-identical positive outputs, one identical rejected diagnostic.
- Quad-native and PR46 follow-up test modules: 30/30 PASS, including deterministic truth table, fail-closed evidence, tamper, import lineage, and 1000-element witness.
- C2=C3=C4 fixed point: pending independent qualification; neither self-hosting nor readiness to merge is implied by the above tests.

## Gate
Require fixed point, frozen verifier, new-base CI, and completed review on the exact commit. Harness authority forbids automatic merge without separate owner GO.
