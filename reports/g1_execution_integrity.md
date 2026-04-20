# G1 Execution Integrity

Status: completed evidence report for `Q3`

## Goal

Test whether the admitted current-`main` execution path preserves meaning across:

- source
- semantics
- IR lowering
- SemCode emission
- verifier admission
- VM execution

This report follows:

- `docs/roadmap/release_qualification/gate1_protocol.md`

## Reproducible Evidence Pack

Representative source fixtures reused from `Q1`:

- `examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm`
- `examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm`
- `examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm`

Canonical harness:

```text
cargo test -q --test g1_execution_integrity
```

The harness goes through the public compiler/runtime surface:

- `check_source(...)`
- `compile_program_to_ir(...)`
- `compile_program_to_semcode(...)`
- `verify_semcode(...)`
- `run_verified_semcode(...)`
- `disasm_semcode(...)`

## Representative Stage Snapshots

Observed stable baseline on current `main`:

```text
program=cli_batch_core
sema:warnings=0 laws=0
ir:names=classify_exit,main
semcode:magic=SEMCOD13 rev=14
verify:names=classify_exit,main
disasm:names=classify_exit,main
run=ok

program=rule_state_decision
sema:warnings=0 laws=0
ir:names=decide,main
semcode:magic=SEMCODE0 rev=1
verify:names=decide,main
disasm:names=decide,main
run=ok

program=data_audit_record_iterable
sema:warnings=0 laws=0
ir:names=__impl::Iterable::Samples::next,main,summarize
semcode:magic=SEMCOD12 rev=13
verify:names=__impl::Iterable::Samples::next,main,summarize
disasm:names=__impl::Iterable::Samples::next,main,summarize
run=ok
```

What this proves:

- the admitted representative programs preserve the same function surface from IR
  through verifier and disasm
- the current executable iterable slice reaches SemCode and VM without semantic
  disappearance
- the public `run_verified_semcode(...)` path stays successful after verifier
  admission

## Negative Execution Evidence

The canonical negative case mutates valid SemCode into malformed function data.

Observed behavior:

- `verify_semcode(...)` rejects before execution
- the rejection contains `InvalidStringTable`
- raw `run_semcode(...)` also rejects the malformed payload

Operational meaning:

- malformed SemCode is not silently admitted into verified execution
- verifier/runtime rejection remains aligned on the malformed-binary path

## Determinism Evidence

For each representative source program, the harness repeats:

- SemCode compilation three times
- disassembly twice
- verified execution three times

Observed behavior:

- SemCode bytes are identical across repeated compiles
- disassembly is stable across repeated reads of identical bytes
- verified execution remains stable across repeated runs

Two real defects were found and fixed while running this qualification step:

1. verifier incorrectly treated `CAP_OWNERSHIP_FIELD_PATHS` as requiring every
   `SEMCOD12/13` program to contain a `Field(SymbolId)` payload
2. `disasm_semcode(...)` emitted functions through unordered `HashMap` iteration,
   which made the snapshot surface nondeterministic even for identical SemCode

Both fixes were narrow and directly tied to Q3 evidence integrity.

## Boundary Notes

The blocked module-based executable program from `Q1` is not counted here as an
execution-integrity failure, because it does not reach the full
`source -> sema -> IR -> SemCode -> verifier -> VM` path on current `main`.

That remains a source/frontend practical-readiness limitation, not evidence of a
semantic-preservation break inside the admitted execution contour.

## Q3 Verdict

`G1-C Execution Integrity` is green for the admitted current execution contour.

Operational verdict:

- source-to-runtime semantic preservation is trusted on the representative
  admitted programs
- malformed SemCode rejection is explicit before verified execution
- pipeline determinism is evidenced on the current representative pack

This does **not** yet upgrade the overall release decision by itself, because
broader practical readiness is still constrained by evidence already found in
`Q1` and `Q2`.
