# SHF-15 Quad-Native Compiler Experiment


> **Architecture correction — 2026-10-30:** See [QUAD_NATIVE_ARCHITECTURE_AUDIT.md](QUAD_NATIVE_ARCHITECTURE_AUDIT.md).
> The earlier declaration-registration trick (`Map(text,quad)`, first declaration T,
> duplicate F, then S) was **semantically invalid and has been removed**.
> The earlier performance and artifact hashes below are historical measurements
> **before** this correction; they are not the hashes of the current working tree.
> The present implementation uses genuine quad return-path evidence in Sema;
> use newly recorded final-test identities for qualification.


Status: **experimental engineering candidate, not a Bootstrap Seal**.

## Authority and isolation

- Candidate branch: `feat/quad-native-shf15`, based on PR #45 HEAD `972066b25d2201344747becff0a0d6abfeb0bf4d`.
- Qualified C0 reference: `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`.
- Normative source semantics: C0 `docs/spec/source_semantics.md`, `quad_logic_frame_v1.md`, and Foundation Source Profile 1.2.
- No changes to `bootstrap/**`, `reference/**`, `.harness/**`, `.github/**` or the runtime.
- The experimental nine-file source bundle is **not** the declared canonical `S` while `bootstrap/source-set.toml` has `files = []`.

## Vertical implementation

1. **Parser:** `quad` type reference, literals `N/F/T/S` (AST kind 9), four-way match patterns (kind 5) and fast-path exclusions to avoid mistaking quad literals for variables.
2. **Sema:** quad type identity 5 with `TyInfo.k = 10`; quad equality; lattice `&&` / `||`; unary `!`; typed quad pattern checks and duplicate-arm rejection. Bare quad `if` remains an error. The C0-required `_` match default remains mandatory.
3. **Compiler-internal quad evidence:** declarations of record and function names accumulate evidence with `Map(text, quad)` and knowledge join. First occurrence: `N || T = T`; duplicate occurrence: `T || F = S`. Diagnostics retain the existing duplicate-name failure path. This is *real* quad computation inside the Semantic-written compiler.
4. **Lower/emit:** direct SemCode `LOAD_Q = 0x01`, `Q_AND = 0x10`, `Q_OR = 0x11`, `Q_NOT = 0x12`, existing `CMP_EQ`. `LOAD_Q` is four bytes, `Q_NOT` five bytes. Quad-typed parameter family is recorded. Terminal exhaustive returning matches have a fail-closed successor instead of an invalid jump to EOF.

## Four-state model

`N=(0,0)`, `F=(0,1)`, `T=(1,0)`, `S=(1,1)`, where the planes are independent truth and falsity evidence.

- Knowledge join `a || b = (a.t | b.t, a.f | b.f)`.
- Knowledge meet `a && b = (a.t & b.t, a.f & b.f)`.
- Negation `!a = (a.f, a.t)`.
- These are **not** the distinct `qtruth_*` truth-table operations.
- Quad values do not implicitly coerce to `bool`.

## Qualification evidence and remaining gates

Run:

```powershell
$env:SEMANTIC_SMC = 'C:\Users\said3\AppData\Local\Programs\Semantic\bin\smc.exe'
python tests/compiler/test_quad_native.py -v
python -m unittest discover -s tests/compiler -p 'test_imports.py' -v
```

The quad-native tests cover all 16 join and 16 meet combinations, four negations, four match outcomes, comparison to C0 program execution, verifier admission, VM execution, and negative diagnostic boundaries for conditions, matches, mixed families and duplicate declarations.

**No completion claim** before: standard regression suite at this exact candidate, same-source C0 compile + candidate recompilation with authorized quota, C1/C2 fixed-point proof, canonical source-set registration under separately authorized governance, and test evidence on the exact final source identity.

This experiment deliberately does **not** weaken or rewrite Bootstrap Contract v2. Existing 20,000,000 Steps / 50,000 Calls / 256 Frames remain the qualification limit. Any additional quad surface must be reconciled with the normative bootstrap subset before a production qualification claim.

## Independently reproduced evidence — 2026-10-30

- Updated compiler source compiled by frozen C0 and admitted by its verifier (38 functions).
- Quad positive program: 16 knowledge-joins, 16 knowledge-meets, four negations, four match outcomes; C0 and candidate artifacts both admitted and executed successfully.
- Six negative program families: bare quad condition, incomplete match, duplicate quad match arm, mixed bool/quad operator, duplicate function, duplicate record.
- Import regression: 5/5 PASS; quad tests: 2/2 PASS.
- Local `scripts/harness-check.ps1`: PASS; `git diff --check`: PASS.
- Experimental `C0(S_experimental)` run under **30,000,000 Steps, 50,000 Calls, 256 Frames**: outcome OK; **23,282,440 Steps; 44,188 Calls; 33 peak frames**.
- Experimental SemCode: **217,814 bytes**, SHA-256 `fead240e5df1ec64b80b6d0f629f98b7d1332a96402f0c67717e6ecfad393e55`; C0 `smc verify`: PASS.
- Delta against the prior optimized 22,216,724-Step experiment is +1,065,716 Steps and +1,376 Calls; this is **not** a throughput improvement.
- Source fixed-point and normative quota are **NOT** passed; 20M Steps exceeded by 3,282,440.
- The experiment does not make the still-empty `bootstrap/source-set.toml` manifest canonical or satisfy the frozen Bootstrap Seal.

### Intentional legacy contract divergence

`tests/compiler/cases/n24_unsupported_type_annotation.sm` declares `fn f(q: quad)`. Legacy `cases.json` expects `reject:unsupported:type 'quad'`, while the pinned C0 `reference_cases.json` already expects `ok`. The experimental quad-native compiler now accepts this valid C0 surface; therefore the legacy 124-case qualification suite will report a **contract-related FAIL** for n24 until the bootstrap subset is separately reauthorized and its expectations reconciled. Do not silently change the frozen bootstrap contract or the legacy expected results to conceal this difference.


## Final verification after terminal-match repair

The source bundle identified by `quad_source_snapshot.json` (experimental aggregate SHA-256 `d6ac587d206799ce97d244375e560d33b7093c55aa13b4e7404088e2daeab08c`) was retested after the terminal jump-to-EOF fix.

- Independent C0 run on the final compiler: `outcome=ok`, **23,308,379 VM Steps**, **44,230 VM Calls**, **33 peak Frames**, under diagnostic `--max-steps 30000000 --max-calls 50000 --max-frames 256`.
- Output: **218,082 bytes**, SHA-256 `1a408be05ea5efffcffd5cfbc24cf2b009f20934fd0b5d27c381b43c2366b5de`.
- C0 `smc verify`: **PASS**, 38 functions, header `SEMCOD23`, epoch 0.24.
- Full old differential suite: **123/124 cases PASS**, **9/9 mutation cases PASS**; only `n24_unsupported_type_annotation` failed because the old subset requires rejection of `quad` despite pinned C0 accepting that source. The suite exits nonzero, so its contractual gate is **NOT passed**.
- New quad-native tests: **3/3 PASS** (16 join cases, 16 meet cases, four negations, four match states, six negative boundaries, terminal unit match).
- Existing import tests: **5/5 PASS** from prior run.
- Working tree `git diff --check`: PASS. Local Harness: PASS.
- Normative quota: **FAIL**; 20,000,000 Steps exceeded by **3,308,379**. `C1 -> C2` fixed-point: **NOT TESTED**. This output is an **experimental C1-like artifact**, not a canonical `C0(S)` under the presently empty source-set manifest.

No commit, push, merge, or governance-contract modification was performed.
