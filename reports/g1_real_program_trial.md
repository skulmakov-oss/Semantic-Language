# G1 Real Program Trial

Status: completed evidence report for `Q1`

## Goal

Test whether current `main` can express actual small programs through the
public executable surface, rather than only isolated feature demos.

This report follows the canonical Gate 1 protocol in:

- `docs/roadmap/release_qualification/gate1_protocol.md`

UI is not part of this qualification contour.

## Reproducible Evidence Pack

Canonical committed trial programs:

- `examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm`
- `examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm`
- `examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm`
- `examples/qualification/g1_real_program_trial/module_helpers_blocked/src/main.sm`

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

- `examples/qualification/g1_real_program_trial/module_helpers_blocked/src/main.sm`

Intent:

- write a small multi-file executable program with a helper module

Observed behavior:

- `smc check` rejects the entry module
- `smc run` rejects the entry module
- current parser surface reports top-level `Import` as invalid in this
  executable program path

Observed blocking surface:

```text
expected top-level 'enum', 'fn', 'impl', 'record', 'schema', 'trait', or role-marked schema declaration
```

Verdict:

- `blocked`

Reason:

- a normal module-based executable helper split is not admitted as a working
  current-`main` program path
- this is a real qualification blocker for broader practical programming claims

## Additional Friction Found During Authoring

One extra limitation showed up while drafting the trial programs:

- direct `i32` accumulation through `+=` produced
  `f64 arithmetic requires f64 operands, got I32 and I32`

That probe is not counted as a formal trial-family program, but it is relevant
evidence:

- integer control/data handling is usable
- integer arithmetic ergonomics still require more trust work before a broader
  practical-readiness claim

## Q1 Summary

Current `main` can already support:

- small single-file utility cores
- rule/state-oriented executable programs
- narrow data programs over `Sequence(T)` and direct-record `Iterable` impls

Current `main` does **not** yet prove:

- ordinary module-based executable program authoring
- full CLI-style practical usability with admitted IO/process interaction

## Q1 Verdict

Gate `G1-D` is now evidenced, but the outcome is mixed rather than fully green.

Per-program verdicts:

- CLI utility core: `tolerable`
- rule/state-oriented program: `natural`
- data-heavy small program: `tolerable`
- module-based program: `blocked`

Operational conclusion:

- Semantic is already capable of writing some real small programs
- Semantic is not yet qualified to claim broad practical-programming readiness
  while ordinary module-based executable authoring remains blocked on current
  `main`
