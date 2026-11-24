# C0 code-generation parity

Status: **ENGINEERING — implemented in `compiler/lower.sm`, `compiler/semcode.sm` and `compiler/driver.sm`.**
Checked by byte comparison against the pinned C0 (`skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`,
`smc compile` at its default `O0`). This is not a normative qualification.

## Why

Bootstrap Contract v3 fixes `C1 = C0(S)`, `C2 = C1(S)` and `C1.smc == C2.smc` under
`byte-equality-v1` ([BOOTSTRAP_CONTRACT.md §10.4](../BOOTSTRAP_CONTRACT.md)). C1 is produced by the
Rust code generator of C0; C2 is produced by the Semantic compiler. The two artifacts can only be equal
if the Semantic compiler emits exactly the bytes C0 emits for the same source.

Before this change the compiler emitted different, equally valid SemCode. For the historical source
`4c563b74`, `G1 = C0(S)` was 324,114 bytes and `G2 = G1(S)` was 301,087 bytes: the set and order of
the 95 functions were the same, but only 7 functions were byte-identical. The differences were
systematic code-generation choices (below), not functional differences. Every rule here was taken from
the C0 source at the pinned SHA and confirmed by comparing bytes.

## Rules

| # | Topic | C0 behaviour (source at the pin) | Before |
|---|---|---|---|
| P1 | Local names | Each declaration binds `__sm_local_<n>_<spelling>`, where `n` counts declarations in the function in lowering order (`LoweredLocalEnv::bind`, `crates/sm-ir/src/legacy_lowering.rs:12197`). | `<spelling>#<slot>` |
| P2 | String table | Strings are interned at their first use, in instruction order. Slot numbers (`LOAD_VAR`/`STORE_VAR`), callee names (`CALL`), record, field and ADT names are indexes into that table. Unused strings do not appear. | Locals first, then the other strings |
| P3 | `if` statement | `JMP_IF cond, then; JMP else`. A branch that ends in `RET` gets no `JMP` to the join (`jump_to_join_unless_returned`, `:12336`). The join label exists only if it is reached. All statements of a block are lowered, including any after a `return`. | `BOOL_NOT` + `JMP_IF else` |
| P4 | `while` | `test: cond; JMP_IF cond, body; JMP end; body: …; JMP test; end:` (`lower_while_stmt`, `:6765`). | Inverted test |
| P5 | `for x in seq` | The sequence stays in its register. Seven registers are allocated up front (zero, one, len, index, cmp, item, next). Only the index is a local, `__for_each_seq_<id>_index` (`lower_for_sequence_stmt_from_reg`, `:6987`). | Sequence spilled to a local, length reloaded every iteration |
| P6 | Label ids | `if`, `while`, `for` and statement `match` take `<id>` from a per-function counter (`next_if_id`, `:12322`). | — |
| P7 | Value `if` | Its id is taken from the **register** counter (`alloc_if_expr_id`, `:11256`). Both arms store `__if_expr_<id>_result` and jump to the join, which reloads it. | `BOOL_NOT` form, unnamed temporary |
| P8 | `match` | Statement position follows `Stmt::Match` (id from `next_if_id`, `:8259`; join only if reached). Expression position follows `Expr::Match` (id from the register counter, `__match_expr_<id>_result`, `:10783`). Each arm is `lit; CMP_EQ; JMP_IF arm; JMP next`. Option matches compare `ADT_TAG`. A `_` arm becomes the default label. An exhaustive Option match without `_` ends in `LOAD_BOOL false; ASSERT` (`lower_impossible_match_trap`, `:9596`). A statement `match` on `quad` without guards uses a dispatch table. | Single shape, `BOOL_NOT`, no ids |
| P9 | Function end | An implicit `RET` is added only when the last instruction is not `RET` (`:1072`). | Structural `always_returns` analysis |
| P10 | OWN0 field symbols | `FieldSymbol` is the C0 arena `SymbolId` of the field name. C0's parser interns identifier spellings in parse order (`AstArena::intern_symbol`, `crates/sm-front/src/types.rs:1414`): type keywords, quad literals, `_` and `Option` in type position are not interned; `assert` is; `Iterable` is interned after each `for` body (`crates/sm-front/src/parser.rs:1416`). The body is the `for` statement's block in the parsed AST; `c0_symbol_ids` takes its `{` offset from there, never from token guessing. `driver.sm` (`c0_symbol_ids`) reproduces this order from the token stream of the expanded source. | Index into a global table built from the function string tables |

The lowering keeps its structure: every rule is expressed in Semantic in the same lowering pass. The
host still only transports bytes. `lower_function(sem, fi)` keeps its signature for the internal test
programs. The driver calls `lower_function_with_symbols`, which carries the C0 symbol table (P10).

## Measured result

- **Compiler corpus:** for all 71 positive programs in `tests/compiler/cases.json` and the positive
  import fixture, the bytes are equal to the C0 compilation of the same source.
  - `a14_match_and_loop_shapes` exercises the `quad` statement-match dispatch, value `match`, an
    Option `match` without `_`, and `while` with `break`/`continue` next to `for`.
  - `a15_for_iterable_record_literal` and `a16_for_iterable_value_if` pin rule P10 when the `for`
    iterable contains braces (a record literal, a value `if`). Both differed from C0 before the
    review fixes; a token heuristic had taken the first `{` as the body. Rejections keep their
  categories and messages.
- **Compiler source:** for `S` itself, `C1 = C0(S)` and `C2 = C1(S)` are byte-identical. Exact
  identities, counters and the pipeline that produces them are in [README.md](README.md).

## Limits

- **Scope of the evidence.** Parity is established for the constructs exercised by the corpus and by
  `S`. The rules mirror C0's lowering for the admitted subset only. A construct outside the corpus
  could still diverge; the differential checker reports such a divergence as a failure, not as a pass.
- **The P10 interning order is reconstructed, not shared.** It is derived from the tokens, matching C0
  on the corpus, on `S` and on probes for imports, type names used as field names, `for`, `assert`
  and Option patterns. Module and import order follow the R1 textual expansion, which matched C0 on the
  probes and on `S`.
- **Resource cost.** C0's shapes are less compact than the previous ones. Self-compilation now takes
  about 100.8M steps and 1.09M calls, against 87.8M before.
