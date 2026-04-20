# G1 Benchmark Baseline

Status: completed evidence report for `Q4`

## Goal

Establish a reproducible measurement baseline for the admitted current-`main`
pipeline instead of relying on intuition.

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Scope

This first baseline measures in-process public pipeline stages for the same
representative executable programs already used in `Q1` and `Q3`:

- small single-file CLI-style core
- medium rule/state program
- data-heavy direct-record iterable program

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
- blocked module-based executable entry, because it is already known not to
  reach the admitted pipeline on current `main`

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
lex_us=min:32 median:55 max:183
parse_us=min:79 median:168 max:577
sema_us=min:226 median:357 max:1965
ir_us=min:277 median:348 max:1323
emit_us=min:331 median:452 max:820
verify_us=min:25 median:30 max:68
runtime_us=min:252 median:421 max:669

scenario=medium_rule_state
path=examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm
snapshot=tokens:226 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:100 semcode_bytes:654 semcode_hash:cb30d87c1081677e
lex_us=min:43 median:92 max:122
parse_us=min:111 median:220 max:255
sema_us=min:506 median:1055 max:2314
ir_us=min:497 median:1226 max:1571
emit_us=min:559 median:1757 max:3318
verify_us=min:30 median:68 max:82
runtime_us=min:166 median:380 max:685

scenario=record_iterable_data
path=examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm
snapshot=tokens:359 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:3 ir_instructions:131 semcode_bytes:1030 semcode_hash:5ca7f7eee779e16f
lex_us=min:130 median:137 max:246
parse_us=min:265 median:374 max:635
sema_us=min:822 median:1178 max:1791
ir_us=min:839 median:1673 max:2266
emit_us=min:839 median:1869 max:2989
verify_us=min:78 median:89 max:164
runtime_us=min:438 median:792 max:1536
```

## Interpretation

What the baseline currently shows:

- stage timings are reproducible enough to support future regression comparison
- the admitted current contour stays small and fast on representative programs
- verifier time is consistently small relative to emit/runtime on this scenario
  set
- the data-heavy record iterable program is the heaviest admitted scenario in
  the current first-cycle pack, as expected

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
