Snake benchmark gap matrix fixtures for `PR-A2`.

Purpose:

- freeze the current pass baseline for already-landed benchmark-critical source
  surfaces
- freeze the current fail baseline for still-missing snake blockers that
  already have a meaningful current source spelling

Current landed positive baseline includes:

- same-family text equality
- enum/control-flow basics
- same-family plain `i32` relational operators
- same-family plain `i32` unary `-` and binary `+`, `-`, `*`
- ordered `Sequence(T)` indexing and iteration
- first-class closure capture

This fixture pack intentionally does not yet freeze syntax for two blocker
families:

- seeded deterministic pseudo-random source
- narrow stdout experiment surface

Reason:

- current `main` and the application-completeness ledger define those blocker
  families, but they do not yet define one canonical source spelling
- this PR must not invent fake API names just to make the matrix look more
  complete

Those two gaps remain part of the benchmark blocker set and should be frozen in
tests only after their scope PRs choose the public source forms.
