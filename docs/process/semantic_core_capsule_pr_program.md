# Semantic Core Capsule PR Program

Status: draft planning baseline

This document is the canonical PR package set for the public Semantic core capsule.

Rule for this line of work:

- package planning comes first
- implementation follows only after the package set is accepted
- each PR stays narrow and traceable
- no package may expand the public surface beyond the stated scope
- reserved extension surfaces stay out of code, help, docs, tests, and comments

## Purpose

Build a deterministic execution core that provides:

- quad algebra and packed execution substrate
- typed instruction execution
- scalar reference behavior
- verifier-facing admission rules
- golden, differential, and tail-length tests
- public-safe local bench and lab tooling

## Normalization Notes

- The detailed package list is the canonical numbering source.
- `CORE-03` is normalized to mask calculus.
- `CORE-04` is normalized to the dual-plane tile layer.
- Package IDs and titles below are the ones to use in branch names, PR titles, and traceability notes.

## Package Rules

Each PR package must include:

- goal
- explicit dependencies
- scope
- acceptance checks
- out-of-scope guardrails

Each implementation PR must also satisfy:

- `cargo fmt`
- touched-crate tests
- no unwanted dependency direction
- updated docs when the public contract changes
- deterministic output for public tools and tests

## Wave Matrix

| Wave | Packages | Outcome |
| --- | --- | --- |
| CORE-00 | `00A`, `00B` | workspace scaffold and public/internal boundary |
| CORE-01 | `01A`, `01B` | frozen quad algebra |
| CORE-02 | `02A`, `02B` | packed 32-lane quad register |
| CORE-03 | `03A`, `03B` | typed mask calculus |
| CORE-04 | `04A`, `04B`, `04C` | dual-plane tile substrate |
| CORE-05 | `05A`, `05B` | state delta calculus |
| CORE-06 | `06A`, `06B` | bank and batch containers |
| CORE-07 | `07A`, `07B` | core value model and fixed-point arithmetic |
| CORE-08 | `08A`, `08B` | opcode set and typed instruction format |
| CORE-09 | `09A`, `09B` | register frame and program model |
| CORE-10 | `10A`, `10B` | traps and fuel |
| CORE-11 | `11A`, `11B` | scalar executor and call support |
| CORE-12 | `12A`, `12B` | backend contract surface |
| CORE-13 | `13A`, `13B`, `13C` | scalar backend and CPU caps scaffold |
| CORE-14 | `14A`, `14B` | verifier-facing admission profile and validation |
| CORE-15 | `15A`, `15B` | SemCode bridge boundary and test builder |
| CORE-16 | `16A`, `16B` | golden vectors and result digest |
| CORE-17 | `17A`, `17B` | differential and tail-length tests |
| CORE-18 | `18A`, `18B` | local benchmark harness |
| CORE-19 | `19A`, `19B` | public-safe lab CLI |
| CORE-20 | `20A`, `20B` | execution docs and wording hygiene |

## PR Packages

### CORE-00A

Title: `core: create capsule workspace skeleton`

Goal:

- introduce the public core capsule as a separate workspace slice

Depends on:

- none

Scope:

- add the core workspace members
- add minimal crate manifests
- add crate roots for capsule, quad, exec, runtime, backend
- keep the first pass buildable with minimal placeholders

Acceptance:

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo check -p semantic-core-quad --no-default-features`

Out of scope:

- executor behavior
- SIMD implementation
- VM bridge

### CORE-00B

Title: `core: define public and internal boundary`

Goal:

- lock the facade shape before implementation detail spreads

Depends on:

- `CORE-00A`

Scope:

- expose a small public capsule facade
- keep internal modules non-exported
- expose only stable public result, config, status, and error types
- ensure backend detail does not appear in crate docs

Acceptance:

- public API is intentionally small
- internal modules are not exported
- generated docs do not expose backend internals
- `cargo doc -p semantic-core-capsule --no-deps` stays clean
- generated doc output is checked for leaked internal names such as `sealed`

Out of scope:

- runtime logic
- validation logic

### CORE-01A

Title: `quad: define QuadState and frozen encoding`

Goal:

- freeze the base four-state algebra

Depends on:

- `CORE-00A`

Scope:

- define `QuadState`
- add bit conversion helpers
- add plane accessors
- add `inverse`, `join`, `meet`, and raw xor
- keep layout stable with `repr(u8)`

Acceptance:

- encoding tests pass
- invalid bit patterns are rejected
- truth tables match the contract

Out of scope:

- packed lanes
- executor integration

### CORE-01B

Title: `quad: add formal truth table tests`

Goal:

- make the algebra executable as tests

Depends on:

- `CORE-01A`

Scope:

- exhaustive join table
- exhaustive meet table
- inverse table
- known, null, and conflict classification tests

Acceptance:

- join matches bitwise OR
- meet matches bitwise AND
- inverse swaps true and false planes

Out of scope:

- benchmarks
- program execution

### CORE-02A

Title: `quad: implement QuadroReg32`

Goal:

- add the compact 32-lane packed quad register

Depends on:

- `CORE-01A`

Scope:

- define `QuadroReg32`
- add raw conversion
- add checked and unchecked lane access
- add packed `join`, `meet`, `inverse`, and `raw_delta`
- freeze the lane masks

Acceptance:

- all lane/state get-set combinations pass
- packed operations match per-lane algebra
- out-of-bounds access is rejected

Out of scope:

- tile layout
- backend dispatch

### CORE-02B

Title: `quad: add reg32 debug and deterministic display`

Goal:

- provide a stable debug format for packed registers

Depends on:

- `CORE-02A`

Scope:

- implement deterministic `Debug`
- avoid backend naming in the output
- avoid allocation in the hot formatting path

Acceptance:

- `Debug` output is deterministic
- the output format is stable and compact

Out of scope:

- serde or text serialization policy

### CORE-03A

Title: `mask: implement QuadMask32`

Goal:

- add typed 32-lane lane masks

Depends on:

- `CORE-02A`

Scope:

- define `QuadMask32`
- enforce valid bit domain
- add expansion to 2-bit lane slots
- add count, emptiness, and iteration

Acceptance:

- invalid upper bits are rejected
- expansion logic matches lane layout
- iterator returns deterministic lane indices

Out of scope:

- tile masks
- mask-based register mutation

### CORE-03B

Title: `quad: implement reg32 masks`

Goal:

- project packed lanes into typed state masks

Depends on:

- `CORE-03A`

Scope:

- define `QuadMasks32`
- expose `known`, `null`, and `conflict`
- add register mask projections
- add masked lane mutation helpers

Acceptance:

- all-uniform state masks pass
- mixed patterns partition correctly
- masked mutation touches only selected lanes

Out of scope:

- state deltas

### CORE-04A

Title: `tile: implement QuadTile128`

Goal:

- add a 128-lane dual-plane tile form

Depends on:

- `CORE-01A`

Scope:

- define `QuadTile128`
- add lane accessors
- add plane access
- add `join`, `meet`, `inverse`, and `raw_delta`
- expose known, conflict, null, true, and false masks
- add mask-based writes

Acceptance:

- all-lane tests pass
- packed tile ops match the scalar algebra
- mask projections are correct

Out of scope:

- backend vectorization

### CORE-04B

Title: `tile: implement QuadMask128`

Goal:

- add typed 128-lane masks for tile operations

Depends on:

- `CORE-04A`

Scope:

- define `QuadMask128`
- add count, emptiness, iteration, and boolean ops

Acceptance:

- iteration and boolean ops are stable
- count matches popcount

Out of scope:

- executor use

### CORE-04C

Title: `tile: implement reg32 to tile128 conversion`

Goal:

- bridge VM-friendly and batch-friendly layouts

Depends on:

- `CORE-02A`
- `CORE-04A`

Scope:

- add `from_regs`
- add `to_regs`
- cover uniform and mixed patterns

Acceptance:

- roundtrip tests pass for all frozen states and a mixed layout

Out of scope:

- transposition kernels

### CORE-05A

Title: `delta: implement StateDelta32`

Goal:

- provide 32-lane transition calculus

Depends on:

- `CORE-03B`

Scope:

- define `StateDelta32`
- derive transition masks from two registers
- cover changed, known, unknown, and conflict transitions

Acceptance:

- exhaustive 4x4 transition tests pass
- no invalid upper-bit leakage

Out of scope:

- tile deltas

### CORE-05B

Title: `delta: implement StateDelta128`

Goal:

- extend transition calculus to tiles

Depends on:

- `CORE-04A`
- `CORE-04B`

Scope:

- define `StateDelta128`
- derive per-tile transition masks

Acceptance:

- per-lane 4x4 transitions pass
- changed and conflict transitions are correct

Out of scope:

- executor hooks

### CORE-06A

Title: `bank: implement QuadroBank`

Goal:

- add an array-backed packed register bank

Depends on:

- `CORE-02A`

Scope:

- define `QuadroBank<const N: usize>`
- keep the register array internal
- add accessors and in-place bulk ops

Acceptance:

- get, set, join, meet, and inverse follow per-register behavior

Out of scope:

- backend dispatch

### CORE-06B

Title: `bank: implement QuadTileBank`

Goal:

- add an array-backed tile bank

Depends on:

- `CORE-04A`

Scope:

- define `QuadTileBank<const N: usize>`
- add accessors and in-place bulk ops

Acceptance:

- tile-bank operations match per-tile direct behavior

Out of scope:

- vector kernels

### CORE-07A

Title: `exec: define CoreValue`

Goal:

- introduce the public value domain for the execution core

Depends on:

- `CORE-01A`

Scope:

- define `CoreValue`
- define `Fx`, `TupleRef`, `RecordRef`, and `AdtRef`
- keep primitive equality behavior stable

Acceptance:

- value sizing is documented by tests
- primitive equality semantics are stable

Out of scope:

- heap object stores

### CORE-07B

Title: `exec: implement Fx Q16.16`

Goal:

- provide deterministic fixed-point arithmetic

Depends on:

- `CORE-07A`

Scope:

- implement checked add, sub, mul, div
- expose raw conversion and comparison

Acceptance:

- arithmetic tests pass
- divide-by-zero traps
- overflow traps

Out of scope:

- transcendental math

### CORE-08A

Title: `instr: define CoreOpcode`

Goal:

- freeze the public opcode vocabulary

Depends on:

- `CORE-07A`

Scope:

- define the opcode enum
- keep discriminants stable
- keep debug names deterministic

Acceptance:

- discriminant tests pass
- no non-public naming appears in the opcode surface

Out of scope:

- compact byte encoding

### CORE-08B

Title: `instr: define instruction format`

Goal:

- add the typed instruction form for in-memory execution

Depends on:

- `CORE-08A`

Scope:

- define the typed `Instr` enum
- cover every public opcode
- keep the format allocation-free per instruction
- freeze the in-memory size with a compile-time assertion

Acceptance:

- typed format supports the full opcode set
- no heap allocation is required per instruction instance
- `Instr` size is fixed and compile-time checked

Out of scope:

- wire format serialization

### CORE-09A

Title: `exec: implement RegId and Frame`

Goal:

- add the register frame model

Depends on:

- `CORE-07A`
- `CORE-08B`

Scope:

- define `RegId`
- define `Frame<const R: usize>`
- initialize registers to `Unit`
- track `pc` and `fuel_used`

Acceptance:

- frame initialization, access, and bounds tests pass

Out of scope:

- call stack

### CORE-09B

Title: `exec: implement Program and Function`

Goal:

- add the typed in-memory program model

Depends on:

- `CORE-08B`
- `CORE-09A`

Scope:

- define `CoreFunction`
- define `CoreProgram`
- validate empty and malformed program shapes at the package level

Acceptance:

- invalid entry and invalid register budget cases are rejected

Out of scope:

- bytecode loader

### CORE-10A

Title: `runtime: define CoreTrap`

Goal:

- freeze the execution trap surface

Depends on:

- `CORE-09B`

Scope:

- define the stable trap enum
- keep trap codes and debug names deterministic
- keep entry-frame `Ret` as normal completion rather than introducing a stack-underflow trap

Acceptance:

- runtime errors map to stable trap codes
- public execution does not rely on panic for runtime failures

Out of scope:

- validator errors

### CORE-10B

Title: `runtime: implement FuelMeter`

Goal:

- make execution bounded and measurable

Depends on:

- `CORE-10A`

Scope:

- define `FuelMeter`
- add consume, remaining, and exhausted checks

Acceptance:

- fuel accounting tests pass
- zero-fuel execution is rejected cleanly

Out of scope:

- weighted per-op fuel schedule

### CORE-11A

Title: `exec: implement scalar instruction dispatch`

Goal:

- build the first working executor

Depends on:

- `CORE-08B`
- `CORE-09B`
- `CORE-10B`

Scope:

- define `CoreExecutor`
- execute loads, quad ops, bool ops, integer ops, fixed-point ops, move, branches, assert, trap, and return
- keep result status deterministic

Acceptance:

- golden programs for scalar execution pass
- trap and fuel cases are deterministic

Out of scope:

- call stack

### CORE-11B

Title: `exec: add call and return support`

Goal:

- extend execution to multi-function programs

Depends on:

- `CORE-11A`

Scope:

- add call frames
- add call depth limit
- define return value convention

Acceptance:

- simple, nested, and recursive depth-limit tests pass
- invalid call targets trap cleanly

Out of scope:

- closures
- coroutine frames

### CORE-12A

Title: `backend: define BackendKind and BackendCaps`

Goal:

- introduce the public backend contract without exposing backend detail

Depends on:

- `CORE-11A`

Scope:

- define `BackendKind`
- define `BackendCaps`
- keep the public surface limited to scalar and auto selection

Acceptance:

- default capabilities resolve to scalar-safe behavior

Out of scope:

- actual SIMD execution

### CORE-12B

Title: `backend: create internal backend trait`

Goal:

- add an internal abstraction point for packed backend behavior

Depends on:

- `CORE-12A`

Scope:

- add an internal backend trait
- cover reg32 and tile bulk operations
- keep the trait non-public

Acceptance:

- trait visibility stays internal
- scalar backend implements the contract

Out of scope:

- public backend plugins

### CORE-13A

Title: `backend: add scalar backend`

Goal:

- codify the scalar reference backend

Depends on:

- `CORE-12B`

Scope:

- implement packed bulk ops for reg32 and tile128
- keep scalar behavior equal to direct operations

Acceptance:

- backend tests match direct packed ops

Out of scope:

- auto-tuning

### CORE-13B

Title: `backend: add x86 feature detection scaffold`

Goal:

- expose standard x86 CPU capability reporting

Depends on:

- `CORE-12A`

Scope:

- detect `popcnt`, `bmi1`, `bmi2`, `avx2`, and `avx512f` behind `std`
- return scalar-safe caps elsewhere

Acceptance:

- x86 builds compile and report capabilities
- non-x86 builds fall back safely

Out of scope:

- vector kernels

### CORE-13C

Title: `backend: add arm feature detection scaffold`

Goal:

- expose standard ARM CPU capability reporting

Depends on:

- `CORE-12A`

Scope:

- detect `neon` and `sve` where supported
- return scalar-safe caps elsewhere

Acceptance:

- aarch64 path compiles
- non-ARM path falls back safely

Out of scope:

- vector kernels

### CORE-14A

Title: `contract: define CoreAdmissionProfile`

Goal:

- introduce verifier-facing structural limits

Depends on:

- `CORE-10B`

Scope:

- define max registers, functions, call depth, instructions, and fuel
- validate zero and excessive limits

Acceptance:

- safe defaults are defined
- zero and unbounded limits are rejected

Out of scope:

- dynamic policy loading

### CORE-14B

Title: `contract: validate CoreProgram`

Goal:

- ensure programs are structurally admissible before execution

Depends on:

- `CORE-14A`
- `CORE-09B`

Scope:

- validate entry
- validate function count and register budgets
- validate jumps and calls
- validate basic return discipline

Acceptance:

- invalid entry, register, jump, and call tests pass

Out of scope:

- semantic type inference

### CORE-15A

Title: `bridge: define SemCode import boundary`

Goal:

- establish the future import boundary without implementing a full loader

Depends on:

- `CORE-14B`

Scope:

- define a source trait or a byte loader stub
- return `UnsupportedFormat` in the first pass

Acceptance:

- the bridge boundary exists
- it does not parse any reserved internal format

Out of scope:

- full SemCode decoding

### CORE-15B

Title: `bridge: add minimal internal bytecode loader for tests`

Goal:

- enable deterministic test program construction without the compiler

Depends on:

- `CORE-15A`

Scope:

- add an internal `CoreProgramBuilder`
- validate builder output

Acceptance:

- simple builder tests pass
- the builder stays out of the stable public API

Out of scope:

- public builder support

### CORE-16A

Title: `tests: add golden core programs`

Goal:

- prove that the executor runs the public instruction core end to end

Depends on:

- `CORE-11B`
- `CORE-14B`

Scope:

- add golden programs for quad, bool, integer, fixed-point, call, and fuel cases
- require a versioned `.core.json` envelope with `format_version`
- run them through the capsule

Acceptance:

- all golden programs pass
- outputs and traps are deterministic
- version mismatches are rejected deterministically by lab tooling and test loaders

Out of scope:

- fuzzer infrastructure

### CORE-16B

Title: `tests: add golden digest checks`

Goal:

- make public execution results digestible and backend-independent

Depends on:

- `CORE-16A`

Scope:

- define `CoreResultDigest`
- include status, return value, trap code, and fuel used
- exclude backend-dependent noise
- keep the digest result-based rather than program-representation-based

Acceptance:

- equal results produce equal digests
- different results produce different digests
- scalar and auto produce equal digests
- renumbering registers inside an equivalent program does not affect the digest unless the observable result changes

Out of scope:

- cryptographic hashing claims

### CORE-17A

Title: `tests: add scalar versus direct quad differential tests`

Goal:

- compare executor quad behavior against direct algebra

Depends on:

- `CORE-16A`

Scope:

- seeded differential tests for `QJoin`, `QMeet`, `QNot`, and `QImpl`

Acceptance:

- 1000 seeded cases per operation pass

Out of scope:

- cross-backend differential tests

### CORE-17B

Title: `tests: add bank tail-length tests`

Goal:

- harden bulk operations against future tail bugs

Depends on:

- `CORE-06A`
- `CORE-06B`

Scope:

- run join, meet, and inverse over a fixed tail-length matrix

Acceptance:

- the full tail-length matrix passes deterministically

Out of scope:

- performance claims

### CORE-18A

Title: `bench: add core benchmark harness`

Goal:

- provide a local-only benchmark shell for packed substrate and execution

Depends on:

- `CORE-13A`
- `CORE-16A`

Scope:

- add `semantic-core-bench`
- expose `quad-reg`, `tile`, `exec`, and `all`
- print deterministic metric keys

Acceptance:

- bench runs locally
- output format is deterministic

Out of scope:

- CI perf gating

### CORE-18B

Title: `bench: add CPU feature report`

Goal:

- expose a machine capability summary for bench interpretation

Depends on:

- `CORE-13B`
- `CORE-13C`

Scope:

- print arch and standard CPU flags
- report selected backend kind

Acceptance:

- report works on x86_64
- fallback report works on non-x86 targets

Out of scope:

- auto backend optimization

### CORE-19A

Title: `cli: add public-safe core-lab runner`

Goal:

- expose a minimal operator CLI for the public core

Depends on:

- `CORE-16A`
- `CORE-18A`

Scope:

- add `run`, `validate`, `caps`, and `bench`
- keep help and error output clean

Acceptance:

- help is clean
- `caps` works
- golden `run` works

Out of scope:

- package manager integration

### CORE-19B

Title: `cli: enforce help hygiene tests`

Goal:

- automatically block wording leaks in the public CLI

Depends on:

- `CORE-19A`

Scope:

- add help, error, and completions hygiene tests
- keep the deny-list in tests

Acceptance:

- all three hygiene tests pass

Out of scope:

- broader repository wording policy

### CORE-20A

Title: `docs: add core execution spec`

Goal:

- document the public execution core and only the public execution core

Depends on:

- `CORE-16A`
- `CORE-19A`

Scope:

- add execution, quad algebra, instruction set, traps and fuel, and backend policy docs

Acceptance:

- docs cover deterministic execution, scalar truth, truth tables, instruction list, trap model, and fuel model

Out of scope:

- future extension internals

### CORE-20B

Title: `docs: add internal comments hygiene pass`

Goal:

- keep wording in the public core neutral and forward-safe

Depends on:

- `CORE-20A`

Scope:

- run wording hygiene over public core code and docs
- replace loaded terms with neutral language such as `internal`, `reserved`, `extended`, or `advanced`

Acceptance:

- repository search across the public core slice returns no forbidden wording hits

Out of scope:

- non-core repository wording

## Recommended Execution Order

1. `CORE-00A`
2. `CORE-00B`
3. `CORE-01A`
4. `CORE-01B`
5. `CORE-02A`
6. `CORE-02B`
7. `CORE-03A`
8. `CORE-03B`
9. `CORE-04A`
10. `CORE-04B`
11. `CORE-04C`
12. `CORE-05A`
13. `CORE-05B`
14. `CORE-06A`
15. `CORE-06B`
16. `CORE-07A`
17. `CORE-07B`
18. `CORE-08A`
19. `CORE-08B`
20. `CORE-09A`
21. `CORE-09B`
22. `CORE-10A`
23. `CORE-10B`
24. `CORE-11A`
25. `CORE-11B`
26. `CORE-12A`
27. `CORE-12B`
28. `CORE-13A`
29. `CORE-13B`
30. `CORE-13C`
31. `CORE-14A`
32. `CORE-14B`
33. `CORE-15A`
34. `CORE-15B`
35. `CORE-16A`
36. `CORE-16B`
37. `CORE-17A`
38. `CORE-17B`
39. `CORE-18A`
40. `CORE-18B`
41. `CORE-19A`
42. `CORE-19B`
43. `CORE-20A`
44. `CORE-20B`

## Minimum Definition Of Done

The core capsule line is considered ready when:

- quad algebra is frozen
- packed reg32 is correct
- tile128 is correct
- mask and delta calculus are correct
- bank and batch layers are correct
- the value model covers the required public primitive types
- the opcode and instruction model are complete
- the scalar executor runs validated programs
- trap and fuel behavior are deterministic
- call and return work
- structural validation is mandatory on the public path
- golden and differential tests pass
- local bench and lab tooling run cleanly
- public docs stay neutral
- `cargo test --workspace` is green
- `cargo check -p semantic-core-quad --no-default-features` is green
