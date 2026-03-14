# PROMETHEUS Gates Specification

Status: draft v0
Owner crate: `prom-gates`

This document defines the current gate registry and binding contract between `prom-abi` host calls and concrete gate endpoints.

## Current Gate Surface

Current canonical gate types:

- `GateId { device_id, port }`
- `GateDescriptor`
- `GateRegistry`
- `GateHostAdapter`
- deterministic mock binding

## Ownership Rule

`prom-gates` owns:

- gate identity and descriptor schema
- registry validation rules
- ABI-facing gate binding adapter
- deterministic gate mocks for tests

`prom-gates` does not own:

- capability policy
- VM execution
- semantic state or rule scheduling
- pulse/event orchestration

## Validation Rule

Current registry rules:

- each `GateId` may be registered once
- read access requires a registered gate descriptor
- write access requires a registered descriptor with write permission
- unknown gate targets must reject before backend dispatch

## Boundary Rule

Current adapter rule:

- `GateRead` and `GateWrite` cross from VM execution into host services through:
  - `prom-abi` host trait boundary
  - `prom-cap` capability checks
  - `prom-gates` registry validation and binding

Current non-goal:

- `prom-gates` does not bind `PulseEmit`; pulse/event routing remains outside the gate registry contract.
