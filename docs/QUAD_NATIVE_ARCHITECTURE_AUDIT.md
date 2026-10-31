# Quad-Native Architecture Audit — SHF-15

**Status:** experimental, open / not a Bootstrap Seal
**Base:** `972066b25d2201344747becff0a0d6abfeb0bf4d` (PR #45)
**Branch:** `feat/quad-native-shf15`
**Normative semantic sources:** pinned C0, `docs/spec/quad_logic_frame_v1.md` and `docs/spec/source_semantics.md`.
**Purpose:** audit whether the Semantic-written compiler actually applies its four-state model, rather than adding `quad` tokens to a boolean implementation.

## 1. Audit finding: false evidence in name registration

**Rejected approach (previous patch):** define first name occurrence as T, second occurrence as F and join to S, then report duplicate. This is logically unsound: the second declaration is not evidence that the first declaration does not exist. Two different predicates, "exists" and "is duplicated", were incorrectly encoded as opposite truth/falsity witnesses **of one proposition**.

**Correction:** record/function duplicate detection is restored to `map_contains` against the canonical namespace tables. This is the correct narrowly deterministic operation until a separately specified source-level evidence/provenance model exists. No "quad everywhere" policy is permitted.

## 2. Real quad-native computation: return-path evidence

The compiler now evaluates **possible control-flow return dispositions** using `return_path_evidence: (Node arena, Node id) -> quad` in `compiler/sema.sm`. The witnessed alternatives refer to *the same return obligation*, not unrelated flags.

| State | Path evidence | Safe to certify mandatory return? |
|---|---|---|
| N | No path outcome witnessed (join identity, unreachable/empty alternative set) | No |
| T | Return witnessed; no fallthrough witnessed | **Yes**, conditional on conservative path coverage |
| F | Fallthrough witnessed; no return witnessed | No |
| S | **Both** return and fallthrough witnessed on alternative paths | No |

State encoding remains exactly `N=00, F=01, T=10, S=11`. The return and fallthrough witnesses are accumulated by the **knowledge join** operator `||`, not ordinary bool OR or a truth-table operator.

Alternative branches `a` and `b`: `evidence(if) = evidence(a) || evidence(b)`. A conditional without an `else` joins the true branch with a conservative `F` witness.

Sequential composition `a; b` follows these rules:

| Evidence after a | Evidence after a; b |
|---|---|
| T | T (b is unreachable) |
| F | evidence(b) |
| S | T `||` evidence(b) (already-returned paths are preserved) |
| N | N (no reachable continuation evidence) |

A `match` merges potential arm outcomes in source order. The first wildcard arm ends dispatch analysis because subsequent arms cannot execute. A match without default adds a fallthrough witness; it cannot establish mandatory return. This preserves frozen C0's first-fit match semantics.

`always_returns` is now a narrow proof gate: **only `evidence == T`** passes. `S` must not be collapsed into success; `N` is not falsehood.

### Important limits — do not overclaim

- This is a **conservative return-path analysis**, not a general solver for all expressions, effects, conditions or loop reachability. Loops currently contribute a fallthrough possibility; this stage does not prove a loop necessarily runs or terminates.
- `quad` encodes **which kinds of evidence are present**, but it does **not** itself store source-level provenance. AST nodes retain source positions, but this accumulator does not yet return independent origin links per witness. A future proof carrier must be explicitly designed for that.
- The compiler still contains ordinary `bool` checks. This is not inherently a defect: structural predicates such as "is map key present?" are binary facts. Converting such predicates into fake `quad` values would make the architecture *less* faithful.
- No C0 semantics, VM opcodes, foundational truth/knowledge tables, frozen Harness, Bootstrap Contract or canonical manifest were changed.

## 3. Semantic compiler surface and opcode lineage

The experimental frontend handles `quad`, literals `N/F/T/S`, `match`, equality, unary `!` and the existing lattice operators `&&` and `||` without implicit `quad -> bool`. The lowering uses the pinned opcodes `LOAD_Q=0x01`, `Q_AND=0x10`, `Q_OR=0x11`, `Q_NOT=0x12`. Lattice and truth-table operation families remain distinct. `match` still requires `_` on this pinned C0.

Do not interpret passing `smc verify` alone as proof of source semantics. Verify complete source → native SemCode → VM behavior against the pinned C0 oracle.

## 4. Test strategy

- **Direct internal witness oracle:** `tests/compiler/programs/quad_flow_internal.sm` imports the real `sema.sm`, constructs explicit AST nodes and tests true/false/mixed/unreachable cases, sequential composition, `if`, `match`, first-fit wildcard ordering and the gate `always_returns == (state == T)`.
- **Quad language frontend:** all 16 knowledge joins, 16 knowledge meets, four negations, four match cases, C0 compile/verify/execute parity, six negative admission families.
- **Legacy imports/regressions:** full source-identity-bound suite, exact candidate. No tests or expected outputs are quietly weakened to make this experimental surface look accepted.
- **Trust boundaries:** `git diff --check`, local Harness; frozen C0 identity separately checked by the suite where supported.
- **Bootstrap Seal blockers:** old `n24_unsupported_type_annotation` intentionally rejects `quad`; normative `max_steps=20M`; empty canonical `source-set.toml`; no C1→C2 proof.

## 5. Admission rule

An implementation qualifies for architectural review only when every quad transition has:
1. **One proposition / interpretation**; explicit meaning of both independent planes.
2. **Concrete positive and negative witnesses**, not invented T/F labels for unrelated metadata.
3. **Merge and transfer laws** that do not erase evidence, especially S.
4. **A deterministic policy** for N and S, including rejection instead of silent bool coercion.
5. **An executable oracle and negative tests**, with provenance gaps explicitly identified.

**Verdict:** the prior declaration-evidence patch did not meet criterion 1. Its replacement, path-disposition evidence, meets the narrowly defined semantic/model checks exercised by the current tests, but is not evidence that the entire compiler is quad-native or that Bootstrap Seal is complete.


## 6. Audit execution and evidence (2026-10-30)

The correction above is applied to the experimental branch. `Map(text,quad)` fabricated duplicate detection was removed. The real `return_path_evidence` uses two witnessed path-outcome planes and `always_returns = (state == T)`.

- Internal AST witness test + quad frontend tests: **4/4 PASS**.
- Existing module import tests: **5/5 PASS**.
- Full original compiler suite: **123/124 cases PASS**, **9/9 mutation checks PASS**; only unchanged legacy `n24_unsupported_type_annotation` fails because it mandates rejection of `quad`, unlike C0.
- Pinned C0 compiles the corrected compiler and its verifier accepts the output: **39 functions**, `SEMCOD23`, epoch 0.24.
- Local Harness and `git diff --check`: PASS.
- Source-snapshot aggregate SHA-256 (experimental, **not** normative source-set hash): `317a149c2eb936383de26bcd768e21d44953a8b892653cf3d225378d378f026f`.
- Self-compile with 30M Steps / 50k Calls: **QUOTA EXCEEDED**, 50,000 Calls used, 18,894,738 Steps when halted; no complete artifact. Do **not** claim normative C1.
- High-quota diagnostic retry uses an isolated process and does not modify Harness governance.

**Architectural judgment:** real quad-native return-path analysis passes its directly exercised cases; a general proof-carrier with source-provenance anchors remains a separate requirement, and bootstrap qualification remains open.


## 7. Completed diagnostic self-compile

Using the **same immutable experimental source snapshot** (`317a149c2eb936383de26bcd768e21d44953a8b892653cf3d225378d378f026f`) and pinned C0, a separate diagnostic execution completed under **60,000,000 Steps / 120,000 Calls / 256 Frames**:

- `outcome = ok`
- **23,704,358 VM Steps**, **51,668 VM Calls**, peak frames **33**
- Output `quad_flow_compiler_C1_diagnostic.smc`: **217,772 bytes**
- Artifact SHA-256: `93fd59abf27f68d936af8579168dc2a88dce8927c26f0aa922ee969c0e5a9cd6`
- Pinned C0 `smc verify`: **PASS** (39 functions; `SEMCOD23`, epoch 0.24)

The **normative** 50,000-Call budget is exceeded by 1,668 calls, and the 20M-Step budget by 3,704,358 steps. The preceding normative-call attempt stopped at exactly 50,000 Calls and 18,894,738 Steps. These diagnostic results are **not** a bootstrap qualification. The source-set manifest is still empty and no C1→C2 fixed-point proof exists.

**Disposition:** keep this worktree isolated and experimental; the quad-native return analysis is demonstrated, but canonical admission, provenance-carrying proof records, quota compliance, and fixed point are separate unclosed gates.
