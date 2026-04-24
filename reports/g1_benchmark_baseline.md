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
- narrow helper-module executable program using direct local-path selected import

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
- out-of-scope executable module forms such as alias, wildcard, re-export,
  package-qualified, or namespace-qualified imports

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
same deterministic direct local-path bare/selected-import bundling rule that
current `smc` uses before semantic checking/lowering.

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
lex_us=min:20 median:30 max:467
parse_us=min:42 median:80 max:402
sema_us=min:143 median:238 max:694
ir_us=min:219 median:333 max:589
emit_us=min:249 median:413 max:732
verify_us=min:18 median:24 max:43
runtime_us=min:172 median:257 max:437

scenario=medium_rule_state
path=examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm
snapshot=tokens:226 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:100 semcode_bytes:654 semcode_hash:cb30d87c1081677e
lex_us=min:29 median:57 max:80
parse_us=min:73 median:111 max:219
sema_us=min:278 median:389 max:663
ir_us=min:370 median:404 max:841
emit_us=min:362 median:450 max:630
verify_us=min:21 median:25 max:44
runtime_us=min:111 median:156 max:253

scenario=record_iterable_data
path=examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm
snapshot=tokens:359 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:3 ir_instructions:131 semcode_bytes:1030 semcode_hash:5ca7f7eee779e16f
lex_us=min:60 median:116 max:160
parse_us=min:121 median:199 max:291
sema_us=min:517 median:655 max:1251
ir_us=min:660 median:779 max:1443
emit_us=min:604 median:944 max:1641
verify_us=min:33 median:39 max:68
runtime_us=min:322 median:391 max:836

scenario=module_helper_entry
path=examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm
snapshot=tokens:57 parsed_functions:2 sema_warnings:0 sema_arena_nodes:0 ir_functions:2 ir_instructions:11 semcode_bytes:114 semcode_hash:6a3934431b5d325b
lex_us=min:14 median:16 max:33
parse_us=min:32 median:37 max:42
sema_us=min:111 median:136 max:203
ir_us=min:109 median:144 max:167
emit_us=min:140 median:171 max:258
verify_us=min:10 median:13 max:163
runtime_us=min:42 median:55 max:84

scenario=selected_import_entry
path=examples/qualification/executable_module_entry/positive_selected_import/src/main.sm
snapshot=tokens:120 parsed_functions:4 sema_warnings:0 sema_arena_nodes:0 ir_functions:4 ir_instructions:31 semcode_bytes:368 semcode_hash:36399bf3e2417796
lex_us=min:29 median:34 max:84
parse_us=min:65 median:70 max:110
sema_us=min:246 median:297 max:323
ir_us=min:274 median:311 max:590
emit_us=min:325 median:345 max:729
verify_us=min:23 median:24 max:27
runtime_us=min:96 median:98 max:144
```

## Interpretation

What the baseline currently shows:

- stage timings are reproducible enough to support future regression comparison
- the admitted current contour stays small and fast on representative programs
- verifier time is consistently small relative to emit/runtime on this scenario
  set
- the data-heavy record iterable program is the heaviest admitted scenario in
  the current first-cycle pack, as expected
- the admitted helper-module executable paths, including selected-import
  bundling, stay small relative to the heavier record-iterable scenario and do
  not introduce a disproportionate pipeline cost on current `main`

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
