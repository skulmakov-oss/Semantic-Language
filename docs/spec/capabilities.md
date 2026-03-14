# PROMETHEUS Capability Specification

Status: draft v0
Owner crate: `prom-cap`

This document defines the current capability contract for PROMETHEUS-facing host calls.

## Current Capability Surface

Current canonical capability kinds:

- `GateRead`
- `GateWrite`
- `PulseEmit`

Current manifest contract:

- schema: `prom.cap.manifest`
- version: `v1`

## Mapping Rule

Current canonical mapping:

- `GateRead` call -> `GateRead` capability
- `GateWrite` call -> `GateWrite` capability
- `PulseEmit` call -> `PulseEmit` capability

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
