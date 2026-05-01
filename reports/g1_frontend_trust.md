# G1 Frontend Trust

Status: completed evidence report for `Q2`

## Goal

Test whether the current source entry surface can be trusted across:

- admitted positive programs
- representative negative programs
- diagnostics for known scope boundaries

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Status Reading

This report uses the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Its role is evidence for the currently admitted frontend contour.

It does not by itself:

- promote landed current-`main` behavior into `published stable`
- define the current release-facing posture
- broaden the current practical-programming claim beyond the evidence pack

## Reproducible Evidence Pack

Canonical positive fixtures:

- `examples/qualification/g1_frontend_trust/positive_sequence_and_match/src/main.sm`
- `examples/qualification/g1_frontend_trust/positive_record_iterable/src/main.sm`
- `examples/qualification/g1_frontend_trust/positive_where_clause/src/main.sm`
- `examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm`
- `examples/qualification/executable_module_entry/positive_selected_import/src/main.sm`

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

### Out-of-scope executable import form

Fixture:

- `negative_top_level_import`

Observed diagnostic:

- contains `top-level executable Import currently admits direct local-path helper-module imports plus selected imports in wave2`

Assessment:

- this is deterministic
- this now reads as an honest boundary diagnostic rather than a parser/source
  failure, because the admitted bare-import helper-module path is already
  working on current `main`

Verdict:

- `trusted`

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
- direct local-path bare and selected executable helper-module imports
- contextual `Option` / `Result` match admission
- negative diagnostics for iterable contract shape, standard-form scope, and
  out-of-scope executable import forms

Still trust-reducing zones:

- broader executable-module authoring remains intentionally narrow because
  top-level alias/wildcard/re-export/package-qualified forms are still out of
  scope on the executable path even though direct local-path selected imports
  are now admitted
- this is no longer a parser trust failure, but it still limits the broader
  practical-programming contour

## Q2 Verdict

`G1-B Frontend Trust` is green for the admitted current contour.

Operational verdict:

- parser/typechecker behavior is stable on the admitted current slices
- diagnostics for several narrow boundaries are explicit and reproducible
- frontend trust for broader practical programming remains limited only because
  the admitted contour itself is still intentionally narrow
