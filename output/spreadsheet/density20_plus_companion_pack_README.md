# Density-20 Plus Companion Pack

This companion pack adds a small set of high-leverage satellite features next to
`Density-20` without changing the core pack itself.

Files:

- `output/spreadsheet/density20_plus_companion_pack.csv`

Why this exists:

- `Density-20` already captures the main low-risk language-density wave.
- The companion pack keeps a few useful extras visible without inflating the
  first package into a broader architecture change.
- It is meant to be imported separately and scheduled only when the main wave
  is under control.

Scope guard:

- Like `Density-20`, this pack stays in `Semantic Core`.
- It should remain inside `sm-front`, `sm-profile`, `sm-sema`, and `sm-ir`,
  with no `prom-*` work.
- `record punning` is intentionally gated on a future canonical record value
  model; it is not a reason to reopen the current narrow-core runtime boundary.

Recommended use order:

1. `D20P-A01` wildcard and discard patterns
2. `D20P-B01` where-clause for local derivations
3. `D20P-C01` let-else
4. `D20P-B02` UFCS and method-call sugar
5. `D20P-D01` record punning

Dependency guidance:

- `wildcard and discard patterns` pairs naturally with `match as expression`,
  `match guards`, and tuple destructuring.
- `where-clause` pairs naturally with block expressions, if/match expressions,
  and expression-bodied functions.
- `let-else` becomes strongest once destructuring bind exists in the main pack.
- `UFCS and method-call sugar` should come only after pipeline and short-lambda
  groundwork is understood.
- `record punning` should remain dormant until records become canonical.

How to use:

1. Import the CSV as a second issue pack, not as a replacement for
   `density20_issue_pack.csv`.
2. Keep milestones or waves separate from the main `Density-20` wave unless a
   specific dependency is ready.
3. Enforce the same rollout discipline as the core pack:
   - spec update
   - formatter rule
   - diagnostics coverage
   - verified-path tests
   - explicit desugaring notes
