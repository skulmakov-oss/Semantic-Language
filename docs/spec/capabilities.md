# PROMETHEUS Capability Specification

Status: draft v0
Owner crate: `prom-cap`

This document defines the current capability contract for PROMETHEUS-facing host calls.

## Current Capability Surface

Current canonical capability kinds:

- `GateRead`
- `GateWrite`
- `PulseEmit`

Current admitted post-stable capability kind:

- `StateQuery`
- `StateUpdate`
- `EventPost`

Current owned planned post-stable capability kinds:

- `ClockRead`

Current `v1` scope rule:

- capability policy is frozen to the narrow host-call family used by the current `v1` ABI
- no wider capability taxonomy is implied for `v1` beyond the calls listed above

Current manifest contract:

- schema: `prom.cap.manifest`
- version: `v1`

## Mapping Rule

Current canonical mapping:

- `GateRead` call -> `GateRead` capability
- `GateWrite` call -> `GateWrite` capability
- `PulseEmit` call -> `PulseEmit` capability
- `StateQuery` call -> `StateQuery` capability
- `StateUpdate` call -> `StateUpdate` capability
- `EventPost` call -> `EventPost` capability

Non-`v1` note:

- capability mapping for `StateQuery` is admitted post-stable without changing
  the narrow `v1` manifest baseline
- capability mapping for `StateUpdate` is also admitted post-stable without
  changing the narrow `v1` manifest baseline
- capability mapping for `EventPost` is also admitted post-stable without
  changing the narrow `v1` manifest baseline
- capability mapping for `ClockRead` remains planned and does not by itself
  widen runtime admission

## Enforcement Rule

Current narrow host boundary rule:

- `sm-vm` may execute host-effect opcodes only through:
  - ABI host trait from `prom-abi`
  - capability check from `prom-cap`

Default denial rule:

- missing capability must reject the effect path before the host call is performed

## Manifest Rule

Current manifest owner:

- `CapabilityManifest`

Current manifest invariants:

- manifest schema must match the current canonical schema id
- manifest version must match the current supported version
- unsupported schema or version must reject before capability enforcement proceeds

## Denial Report Rule

Current denial contract:

- missing capability returns an explicit denial report rather than a boolean
- the denial report must include:
  - denied capability
  - host call identity when enforcement happens on a host call path
  - manifest schema/version metadata
  - denial code and human-readable message
