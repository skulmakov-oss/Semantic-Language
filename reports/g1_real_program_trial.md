# G1 Real Program Trial

Status: completed evidence report for `Q1`

## Goal

Test whether current `main` can express actual small programs through the
public executable surface, rather than only isolated feature demos.

This report follows the canonical Gate 1 protocol in:

- `docs/roadmap/release_qualification/gate1_protocol.md`

UI is not part of this qualification contour.

## Status Reading

This report uses the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Its role is evidence for the current practical-programming contour.

It does not by itself:

- promote landed current-`main` behavior into `published stable`
- override the current release-facing posture in `docs/roadmap/v1_readiness.md`
- replace the current qualification verdict in
  `reports/g1_release_scope_statement.md`

## Reproducible Evidence Pack

Canonical committed trial programs:

- `examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm`
- `examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm`
- `examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm`
- `examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm`
- `examples/qualification/executable_module_entry/positive_selected_import/src/main.sm`

Canonical reproducible harness:

```text
cargo test -q --test g1_real_program_trial
```

The harness runs the public `smc` command surface through `smc_cli::run(...)`
for `check` and `run`, rather than using private compiler shortcuts.

## Trial Matrix

### 1. CLI utility core

Path:

- `examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm`

Intent:

- emulate a batch/CLI classification core over argv-like numeric inputs
- consume a `Sequence(i32)`
- produce a `text` status

Observed behavior:

- `smc check` passes
- `smc run` passes
- the program shape is viable as a small single-file utility core

Verdict:

- `tolerable`

Reason:

- the logic itself is straightforward
- the lack of admitted argv/stdout/file IO means this is only the core of a CLI
  tool, not a full user-facing CLI application

### 2. Rule/state-oriented program

Path:

- `examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm`

Intent:

- model a small decision engine over a nominal record snapshot
- use `quad`, `Result(T, E)`, and explicit match-based settlement

Observed behavior:

- `smc check` passes
- `smc run` passes
- the source reads naturally for the intended domain

Verdict:

- `natural`

Reason:

- this is close to the problem shape Semantic already favors
- record + quad + explicit result handling compose without visible friction

### 3. Data-heavy small program

Path:

- `examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm`

Intent:

- scan a direct-record dataset through the current executable `Iterable` slice
- produce an immutable record summary
- patch that summary with record `with { ... }`

Observed behavior:

- `smc check` passes
- `smc run` passes
- direct-record `Iterable` impl dispatch works end to end on current `main`

Verdict:

- `tolerable`

Reason:

- the admitted slice is real and executable
- the program is writable without hacks
- the surface still feels narrow because only the direct-record iterable form is
  available; broader collection/generic reuse is not there yet

### 4. Module-based program

Path:

- `examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm`
- `examples/qualification/executable_module_entry/positive_selected_import/src/main.sm`

Intent:

- write a small multi-file executable program with a helper module

Observed behavior:

- `smc check` passes
- `smc run` passes
- direct local-path bare and selected helper-module imports now execute through the full
  `source -> sema -> IR -> SemCode -> verifier -> VM` path on current `main`

Verdict:

- `tolerable`

Reason:

- ordinary helper-module authoring now works without hidden compiler shortcuts
- the admitted slice is still narrow because top-level alias, wildcard,
  re-export, and package-qualified executable imports remain out of scope

## Additional Friction Found During Authoring

Two extra limitations showed up while drafting the trial programs:

- direct `i32` accumulation through `+=` produced
  `f64 arithmetic requires f64 operands, got I32 and I32`
- the top-level alias helper-module variant in
  `examples/qualification/executable_module_entry/negative_alias_import/src/main.sm`
  still rejects with:

```text
top-level executable Import currently admits direct local-path helper-module imports plus selected imports in wave2; alias, wildcard, re-export, and package-qualified import forms remain out of scope
```

Those probes are not counted as separate formal trial-family programs, but they
are relevant evidence:

- integer control/data handling is usable
- integer arithmetic ergonomics still require more trust work before a broader
  practical-readiness claim
- executable module authoring is now admitted only for the narrow direct
  local-path bare/selected-import slice

## Q1 Summary

Current `main` can already support:

- small single-file utility cores
- rule/state-oriented executable programs
- narrow data programs over `Sequence(T)` and direct-record `Iterable` impls
- narrow helper-module executable programs using direct local-path bare imports
- narrow helper-module executable programs using direct local-path selected
  imports over function-only helper modules

Current `main` does **not** yet prove:

- broader executable module authoring beyond the direct local-path bare/selected-import
  slice
- full CLI-style practical usability with admitted IO/process interaction

Per-program verdicts for the admitted trial families:

- CLI utility core: `tolerable`
- rule/state-oriented program: `natural`
- data-heavy small program: `tolerable`
- module-based program: `tolerable`

## Q1 Verdict

`G1-D Real Program Trial` is green for the widened admitted contour, but the
contour remains narrow.

Operational conclusion:

- Semantic is already capable of writing some real small programs
- Semantic is now also capable of ordinary helper-module executable authoring
  through direct local-path bare imports and direct local-path selected imports
- Semantic is not yet qualified to claim broad practical-programming readiness
  because executable-module authoring remains narrow and full CLI-style
  practicality is still outside the admitted contour
