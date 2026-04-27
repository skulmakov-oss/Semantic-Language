# GAP: Record Executable Readiness for Canonical Programs

Status: implementation-scope draft  
Track: Semantic Full Readiness / stable language surface  
Source trigger: initial snake/weather program trials

## Problem

Canonical practical programs naturally want `record` values for state, sensor packets, and domain objects. In the weather-station and snake trials, record-based source was not reliable enough for the current executable example path.

This creates a mismatch between the documented language surface and the practical canonical examples that should prove the language.

## Goal

Close the minimal executable record path needed for canonical non-UI programs.

## Required minimal surface

```semantic
record SensorState {
    temp: f64,
    pressure: f64,
}

fn read_temp(s: SensorState) -> f64 {
    return s.temp;
}

fn main() {
    let s: SensorState = SensorState { temp: 28.5, pressure: 1013.0 };
    return;
}
```

## Scope

Define and implement the minimal record path across:

- parser/source surface;
- semantic analysis/type checking;
- IR lowering;
- SemCode emission;
- verifier admission;
- VM runtime values;
- diagnostics;
- canonical examples.

## Required behavior

Support at minimum:

- record declaration;
- record literal construction with explicit fields;
- field access;
- passing record values into functions;
- returning record values from functions if already admitted, otherwise explicitly defer;
- deterministic diagnostics for missing/extra/duplicate fields;
- deterministic field ordering policy.

## Required tests

Positive tests:

- record declaration;
- record literal;
- field access;
- function parameter record access;
- nested non-recursive record only if already admitted.

Negative tests:

- unknown record type;
- missing field;
- extra field;
- duplicate field;
- wrong field type;
- unsupported record return if deferred.

## Acceptance criteria

- a small canonical program can use records without falling back to parallel scalar arguments;
- record source behavior is documented in syntax/types/source semantics;
- emitted SemCode and verifier behavior are version/capability-aware where required;
- VM behavior is deterministic;
- examples no longer need to avoid records purely because the executable path is incomplete.

## Out of scope

- mutable record update / copy-with unless already separately admitted;
- record destructuring;
- record let-else;
- record punning;
- record ownership beyond the currently admitted direct field path;
- schema-to-record generation.

## Validation commands

```powershell
cargo test -q
cargo test -q -p sm-front
cargo test -q -p sm-sema
cargo test -q -p sm-ir
cargo test -q -p sm-verify
cargo test -q -p sm-vm
smc check examples/canonical/weather_station_record/weather_station_record.sm
```
