# G1 Benchmark Baseline

Status: completed evidence report for `Q4`

## Goal

Establish a reproducible measurement baseline for the admitted current-`main`
pipeline instead of relying on intuition.

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Status Reading

This report uses the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Its role is benchmark evidence for the currently admitted qualification contour.

It does not by itself:

- promote landed current-`main` behavior into `published stable`
- widen the current practical-programming claim
- override the release-facing posture in `docs/roadmap/v1_readiness.md`

## Scope

This first baseline measures in-process public pipeline stages for the same
representative executable programs already used in `Q1` and `Q3`:

- small single-file CLI-style core
- medium rule/state program
- data-heavy direct-record iterable program
- narrow helper-module executable program using direct local-path bare import

Measured stages:

- lex
- parse
- semantic check
- IR lowering
- SemCode emission
- verifier admission
- verified VM execution

This baseline does **not** include:

- UI, which remains outside the current qualification contour
- cache cold/warm behavior, because this first pass is intended to isolate the
  compiler/runtime pipeline rather than the filesystem cache path
- out-of-scope executable module forms such as selected or alias imports,
  because they are still intentionally outside the admitted contour

## Reproducible Harness

Canonical harness:

```text
cargo test -q --test g1_benchmark_baseline -- --nocapture
```

The harness measures the public in-process APIs directly:

- `lex(...)`
- `parse_program_with_profile(...)`
- `check_source_with_profile(...)`
- `compile_program_to_ir(...)`
- `compile_program_to_semcode(...)`
- `verify_semcode(...)`
- `run_verified_semcode(...)`

For the admitted helper-module executable slice, the harness first applies the
same deterministic direct local-path bare-import bundling rule that current
`smc` uses before semantic checking/lowering.

Method:

- `1` warmup run per scenario
- `7` measured runs per scenario
- report `min / median / max` in microseconds
- assert deterministic pipeline snapshots across all measured runs

## Environment Note

Baseline capture environment:

- OS: Windows 11 Pro Insider Preview `10.0.26300`, `64-bit`
- CPU: Intel Core i5-9300H, `8` logical processors, max clock `2400 MHz`
- RAM: `21339590656` bytes installed
- toolchain: `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- host target: `x86_64-pc-windows-msvc`

These numbers are a local baseline, not a portability promise.

## Observed Baseline

Harness output on the capture machine:

```text
warmup_runs=1
measured_runs=7

scenario=small_cli_core
path=examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm
snapshot=tokens:156 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:90 semcode_bytes:601 semcode_hash:416f22cbb9708ff7
lex_us=min:28 median:38 max:84
parse_us=min:61 median:92 max:208
sema_us=min:221 median:327 max:579
ir_us=min:254 median:377 max:719
emit_us=min:321 median:400 max:741
verify_us=min:24 median:26 max:37
runtime_us=min:252 median:254 max:486

scenario=medium_rule_state
path=examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm
snapshot=tokens:226 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:100 semcode_bytes:654 semcode_hash:cb30d87c1081677e
lex_us=min:39 median:45 max:143
parse_us=min:106 median:143 max:282
sema_us=min:364 median:485 max:995
ir_us=min:441 median:618 max:1244
emit_us=min:497 median:539 max:1273
verify_us=min:27 median:28 max:57
runtime_us=min:150 median:155 max:386

scenario=record_iterable_data
path=examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm
snapshot=tokens:359 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:3 ir_instructions:131 semcode_bytes:1030 semcode_hash:5ca7f7eee779e16f
lex_us=min:63 median:110 max:219
parse_us=min:179 median:208 max:427
sema_us=min:604 median:669 max:1256
ir_us=min:694 median:857 max:1194
emit_us=min:783 median:825 max:1344
verify_us=min:38 median:40 max:51
runtime_us=min:425 median:432 max:620

scenario=module_helper_entry
path=examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm
snapshot=tokens:57 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:11 semcode_bytes:114 semcode_hash:6a3934431b5d325b
lex_us=min:9 median:10 max:24
parse_us=min:20 median:21 max:54
sema_us=min:70 median:72 max:209
ir_us=min:77 median:78 max:217
emit_us=min:87 median:88 max:242
verify_us=min:6 median:6 max:15
runtime_us=min:27 median:27 max:69
```

## Interpretation

What the baseline currently shows:

- stage timings are reproducible enough to support future regression comparison
- the admitted current contour stays small and fast on representative programs
- verifier time is consistently small relative to emit/runtime on this scenario
  set
- the data-heavy record iterable program is the heaviest admitted scenario in
  the current first-cycle pack, as expected
- the admitted helper-module executable path sits close to the small single-file
  core and does not introduce a disproportionate pipeline cost on current
  `main`

What it does **not** show:

- large-program scaling
- cache effectiveness
- UI startup or event-loop latency
- broader practical-programming performance beyond the admitted current pack

## Q4 Verdict

`G1-E Benchmark Baseline` is green for the current qualification contour.

Operational verdict:

- the project now has a committed, reproducible first benchmark harness
- representative pipeline timings are captured and documented
- future regressions can be compared against a concrete baseline instead of
  memory or intuition
