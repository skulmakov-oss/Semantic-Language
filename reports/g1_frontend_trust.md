# G1 Frontend Trust

Status: completed evidence report for `Q2`

## Goal

Test whether the current source entry surface can be trusted across:

- admitted positive programs
- representative negative programs
- diagnostics for known scope boundaries

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Reproducible Evidence Pack

Canonical positive fixtures:

- `examples/qualification/g1_frontend_trust/positive_sequence_and_match/src/main.sm`
- `examples/qualification/g1_frontend_trust/positive_record_iterable/src/main.sm`
- `examples/qualification/g1_frontend_trust/positive_where_clause/src/main.sm`

Canonical negative fixtures:

- `examples/qualification/g1_frontend_trust/negative_top_level_import/src/main.sm`
- `examples/qualification/g1_frontend_trust/negative_iterable_contract/src/main.sm`
- `examples/qualification/g1_frontend_trust/negative_adt_iterable_scope/src/main.sm`
- `examples/qualification/g1_frontend_trust/negative_result_context/src/main.sm`
- `examples/qualification/g1_frontend_trust/negative_option_match_exhaustiveness/src/main.sm`

Canonical harness:

```text
cargo test -q --test g1_frontend_trust
```

The harness uses the public `smc check` path through `smc_cli::run(...)`.

## Positive Coverage

### Sequence + Option/Result match

Fixture:

- `positive_sequence_and_match`

What it proves:

- `Sequence(i32)` loop admission is stable
- exhaustive `Option::Some/None` match admission is stable
- standard executable assertions are accepted on this path

Verdict:

- `trusted`

### Direct record `Iterable` loop

Fixture:

- `positive_record_iterable`

What it proves:

- trait parsing
- `impl` parsing
- `self: Self` contract admission
- direct-record iterable loop admission

Verdict:

- `trusted`

### Where-clause expression sugar

Fixture:

- `positive_where_clause`

What it proves:

- the current where-clause frontend path is admitted and stable on a normal
  numeric example

Verdict:

- `trusted`

## Negative Coverage

### Module/import executable entry

Fixture:

- `negative_top_level_import`

Observed diagnostic:

- contains `expected top-level`

Assessment:

- this is deterministic
- this is also a real trust problem, because ordinary module-based executable
  authoring is still blocked at the parser boundary on current `main`

Verdict:

- `deterministic but trust-reducing`

### Wrong explicit `Iterable` contract

Fixture:

- `negative_iterable_contract`

Observed diagnostic:

- contains `fn next(self: Self, index: i32) -> Option(Item)`

Assessment:

- the frontend rejects the wrong contract for the right reason
- the message points to the admitted executable shape rather than failing
  generically

Verdict:

- `trusted`

### ADT iterable dispatch out of scope

Fixture:

- `negative_adt_iterable_scope`

Observed diagnostic:

- contains `direct record impls only`

Assessment:

- the frontend does not silently widen iterable dispatch beyond the admitted
  slice

Verdict:

- `trusted`

### Contextless `Result::Ok`

Fixture:

- `negative_result_context`

Observed diagnostic:

- contains `Result::Ok currently requires contextual Result(T, E) type in v0`

Assessment:

- this is a clear, intentional rejection rather than a random type failure

Verdict:

- `trusted`

### Non-exhaustive `Option` match

Fixture:

- `negative_option_match_exhaustiveness`

Observed diagnostic:

- contains `non-exhaustive match expression for Option(T); missing variants: None`

Assessment:

- exhaustiveness diagnostics are explicit and understandable on current `main`

Verdict:

- `trusted`

## Frontend Trust Summary

Trusted zones on current `main`:

- sequence-loop admission
- direct-record iterable trait/impl admission
- where-clause source sugar
- contextual `Option` / `Result` match admission
- negative diagnostics for iterable contract shape and standard-form scope

Still trust-reducing zones:

- ordinary module/import executable entry remains blocked by the current parser
  top-level contract
- this is deterministic, but it means source-level modular authoring is not yet
  trustworthy as a practical executable path

## Q2 Verdict

`G1-B Frontend Trust` is partially evidenced, not fully green.

Operational verdict:

- parser/typechecker behavior is stable on the admitted current slices
- diagnostics for several narrow boundaries are explicit and reproducible
- frontend trust for broader practical programming remains limited while
  module-based executable authoring is still blocked on current `main`
