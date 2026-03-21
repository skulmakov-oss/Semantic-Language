# Density-20 Issue Pack

This pack translates the proposed first language-evolution slice into a
GitHub-friendly backlog artifact tied to the actual crate map in this repo.

Files:

- `output/spreadsheet/density20_issue_pack.csv`

Columns:

- `WBS ID`: stable work-breakdown identifier
- `Wave`: recommended merge wave
- `Issue`: canonical issue title
- `Priority`: recommended priority band
- `Owner Crate`: lead crate that owns the semantic contract for the change
- `Touch Crates`: expected cross-crate implementation surface
- `Acceptance Criteria`: one-line definition of done
- `PR Split`: recommended internal PR staging for the issue

Scope guard:

- `Density-20` is intentionally a `Semantic Core` package.
- It should stay inside `sm-front`, `sm-profile`, `sm-sema`, `sm-ir`, and only
  touch `sm-runtime-core` / `sm-vm` where the value model actually expands.
- No row in this pack assumes changes in `prom-abi`, `prom-cap`, `prom-state`,
  `prom-rules`, `prom-runtime`, or `prom-audit`.

Recommended wave order:

1. `Wave A - Expression Core`
2. `Wave B - Flow Density`
3. `Wave C - API and Data Density`
4. `Wave D - Compile-time Contract Density`

Recommended first `P0` slice if the package must start smaller:

- `D20-A01` Block expressions
- `D20-A02` if as expression
- `D20-A03` match as expression
- `D20-A04` match guards
- `D20-A05` guard clause
- `D20-B03` pipeline operator `|>`
- `D20-B04` expression-bodied functions
- `D20-C06` compound assignment

Owner mapping rationale:

- `sm-sema` leads when the change is fundamentally a source-language semantic
  rule.
- `sm-front` leads when the change is mostly surface syntax / literal form.
- `sm-ir` leads when the feature is primarily sugar over existing execution
  semantics.
- `sm-runtime-core` leads only when the value model truly expands, for example
  tuples.

How to use:

1. Convert each CSV row into one GitHub issue.
2. Use `Wave` as the milestone or iteration grouping.
3. Split implementation using the `PR Split` column rather than landing the
   whole feature as one opaque branch.
4. Keep `prom-*` untouched unless a later architecture decision explicitly
   broadens the scope.
