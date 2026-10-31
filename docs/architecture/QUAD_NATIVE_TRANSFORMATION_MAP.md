# Quad-Native Transformation Map — Q0

Status: experimental inventory for `feat/quad-native-shf15`.

This map records where four-state evidence is semantically required and where a
binary predicate is correct. It does not authorize a blanket replacement of
`bool`, nor does it claim Bootstrap Contract or C1/C2 completion.

## Authority and baseline

- Candidate base: `972066b25d2201344747becff0a0d6abfeb0bf4d`
- Frozen reference: `skulmakov-oss/Semantic@1d1e76c7509a03baa0133abaaa5819f5d32582b2`
- Primary contracts: `docs/spec/quad_logic_frame_v1.md` and
  `docs/spec/source_semantics.md`
- Existing candidate changes are preserved; this file is an inventory only.

## State model

For one proposition, `N/F/T/S` retain independent absence, negative, positive,
and conflicting evidence. Knowledge `||` and `&&` are not truth OR/AND. A
decision is made only by an explicit policy; in the current return proof only
`T` certifies a mandatory return.

## Inventory

| Surface | Current state | Proposition / domain | Required model | Risk | Validation |
|---|---|---|---|---|---|
| `sema.sm:return_path_evidence` | Implemented in candidate | Whether a control-flow path returns | `quad` knowledge evidence | `S` or `N` must not pass | Internal witness fixture; C0 verify/run |
| `sema.sm:always_returns` | Implemented in candidate | Whether return obligation is proven | Explicit `state == T` policy | Collapsing `S` to success is unsound | Return-path and terminal-match tests |
| Return/ fallthrough merge | Implemented in candidate | Same return obligation across alternatives | Knowledge join | Losing either witness hides conflict | `if`, `match`, sequential-path cases |
| `sema.sm:quad` values and patterns | Implemented in candidate | Four-state source value | Native quad type/literals | Implicit bool coercion | 4x4 joins/meets, negation, match |
| `sema.sm:seen` maps | Binary by design | Whether a concrete name/arm was already seen | `bool` membership predicate | Fabricating opposite evidence would be invalid | Duplicate declaration/arm negatives |
| `lexer.sm` scan flags | Binary by design | Lexical mode or byte classification | `bool` | Treating lexer mode as epistemic state is misleading | Boundary and malformed-input tests |
| `parser.sm` lookahead flags | Binary by design | Structural parse condition | `bool` | Broad conversion adds noise, not meaning | Parser regression corpus |
| `main.sm` import flags | Binary by design | Concrete preprocessing facts | `bool` | Import absence is not negative evidence about a proposition | Import suite |
| `lower.sm` option/type flags | Binary by design | Concrete representation/type predicate | `bool` | Artificial quad values could alter lowering | SemCode differential tests |
| `semcode.sm` ownership presence | Binary by design | Whether a concrete section/event exists | `bool`/length predicate | Confusing metadata absence with semantic falsity | Verifier and OWN0 tests |

## Q0 decisions

1. Keep the existing return-path transformation as the first independently
   testable quad-native slice; do not add another accumulator.
2. Keep structural membership, lexer state, parser lookahead, import state and
   representation checks binary. They describe concrete predicates, not
   unresolved evidence about one proposition.
3. Do not transform type errors, duplicate declarations or invalid programs
   into `F`/`S`; diagnostics and epistemic conflict are separate concepts.
4. Provenance remains a gap: the current return accumulator preserves the two
   witness planes but not independent source-origin records for each witness.

## Compatibility and qualification risks

- Quad support must remain aligned with the pinned C0; the legacy bootstrap case
  rejecting `quad` is a contract divergence requiring separate reconciliation.
- The current return analysis is conservative for loops and does not prove
  termination merely because a loop is present.
- Passing verifier admission does not by itself establish semantic equivalence;
  positive, negative, conflict and deterministic artifact tests are required.
- No change here promotes the candidate to C1/C2, a fixed point or a Bootstrap
  Seal.

## Q0 gate

Every proposed conversion above has an explicit proposition/domain and a
validation route. The only conversion selected for the first implementation
slice is the already-present `return_path_evidence` path analysis.
