# SemCode Specification

Status: draft v0
Current format owner: `sm-ir`
Current producer facade: `sm-emit`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`

## Purpose

SemCode is the binary contract between the Semantic producer pipeline and the
Semantic VM.

Ownership rule:

- `sm-ir` owns the SemCode header, opcode, and capability contract in the current `v1` baseline
- `sm-emit` exposes producer-facing entrypoints over that contract and is not a second format owner
- `sm-emit` must re-export the canonical format surface from `sm-ir` rather than maintain a forked local copy

Standard execution rule:

`source -> AST -> sema -> IR -> SemCode -> verify -> execute`

The VM is not the primary structural admission gate.
`sm-verify` is the required admission stage for standard SemCode execution.

## Versioned Header Family

Current supported header family:

- `SEMCODE0`
- `SEMCODE1`
- `SEMCODE2`
- `SEMCODE3`
- `SEMCODE4`
- `SEMCODE5`
- `SEMCODE6`
- `SEMCODE7`
- `SEMCODE8`
- `SEMCODE9`

Observed runtime support in the current toolchain:

- `SEMCODE0`: epoch `0`, revision `1`
- `SEMCODE1`: epoch `0`, revision `2`
- `SEMCODE2`: epoch `0`, revision `3`
- `SEMCODE3`: epoch `0`, revision `4`
- `SEMCODE4`: epoch `0`, revision `5`
- `SEMCODE5`: epoch `0`, revision `6`
- `SEMCODE6`: epoch `0`, revision `7`
- `SEMCODE7`: epoch `0`, revision `8`
- `SEMCODE8`: epoch `0`, revision `9`
- `SEMCODE9`: epoch `0`, revision `10`

Header responsibilities:

- identify the format family
- identify the supported epoch and revision
- carry the emitted capability bitset for the produced artifact

## Version Policy

Compatibility rules:

1. A producer must emit exactly one supported SemCode header variant.
2. A verifier must reject artifacts with unknown or unsupported headers.
3. A VM must not silently reinterpret an unsupported header as a supported one.
4. Any incompatible binary layout or meaning change requires a version bump.

## Current Header Semantics

`SEMCODE0`

- baseline SemCode contract
- does not imply floating-point math capability

`SEMCODE1`

- promoted contract used when emitted program usage requires the `f64` math
  family
- carries the stronger capability envelope required by that produced artifact

`SEMCODE2`

- promoted contract used when emitted program usage requires the canonical `fx`
  value family
- extends the supported opcode/header family without changing standard
  admit-then-execute rules

`SEMCODE3`

- promoted contract used when emitted program usage requires canonical plain
  `fx` arithmetic
- keeps the earlier `SEMCODE2` fixed-point value/equality contract intact for
  older artifacts

`SEMCODE4`

- promoted contract used when emitted program usage requires admitted
  post-stable `StateQuery` host calls
- keeps `SEMCODE0..3` fixed for older artifacts that do not use the widened
  host-call family

`SEMCODE5`

- promoted contract used when emitted program usage requires admitted
  post-stable `StateUpdate` host calls
- keeps `SEMCODE0..4` fixed for older artifacts that do not use the widened
  write-side host-call family

`SEMCODE6`

- promoted contract used when emitted program usage requires admitted
  post-stable `EventPost` host calls
- keeps `SEMCODE0..5` fixed for older artifacts that do not use the widened
  event-side host-call family

`SEMCODE7`

- promoted contract used when emitted program usage requires admitted
  post-stable `ClockRead` host calls
- keeps `SEMCODE0..6` fixed for older artifacts that do not use the widened
  clock-query host-call family

`SEMCODE8`

- promoted contract used when emitted program usage requires the canonical text
  value carrier for admitted literal/equality programs
- keeps `SEMCODE0..7` fixed for older artifacts that do not use executable
  text values

`SEMCODE9`

- promoted contract used when emitted program usage requires the canonical
  ordered sequence carrier for the admitted `M8.3` first-wave surface
- keeps `SEMCODE0..8` fixed for older artifacts that do not use executable
  sequence values

Important rule:

- header selection is derived from actual emitted usage, not from profile
  permission alone

That means:

- a profile may allow `f64`
- if the program does not actually use the `f64` family, the producer may still
  emit `SEMCODE0`

## Capability Contract

The current capability contract is carried by the SemCode header and verified
against actual opcode usage.

Current canonical capability families:

- `CAP_F64_MATH`
- `CAP_FX_VALUES`
- `CAP_FX_MATH`
- `CAP_GATE_SURFACE`
- `CAP_STATE_QUERY`
- `CAP_STATE_UPDATE`
- `CAP_EVENT_POST`
- `CAP_CLOCK_READ`
- `CAP_TEXT_VALUES`
- `CAP_SEQUENCE_VALUES`
- `CAP_DEBUG_SYMBOLS`

Contract rule:

- profile policy constrains what may be produced
- SemCode header records what was actually produced
- verifier proves that opcode usage matches the emitted capability contract

## Structural Contract

Current SemCode admission validates:

- header magic and supported version
- section and function-layout integrity
- opcode validity
- operand shape validity
- jump-target validity
- call-target validity
- register-budget validity against the runtime contract
- string and debug reference validity
- capability consistency between actual usage and emitted contract

## Backward Compatibility Rule

The following changes require a SemCode version review:

- header layout change
- section layout change
- opcode encoding change
- capability bit meaning change
- verifier interpretation change that alters what previously valid artifacts
  mean

Required follow-up:

1. update this specification
2. update verifier compatibility tests
3. update VM compatibility tests
4. update golden or compatibility fixtures if public behavior changed

## No Silent Mutation Rule

The following are forbidden without a documented version change:

- repurposing an existing capability bit
- changing the meaning of an existing header family
- changing section interpretation while keeping the same public version

## Consumer Rule

`sm-vm` may consume SemCode on the standard execution route only through a
verified admission path.

Any raw or testing-only path must not redefine the public SemCode contract.
