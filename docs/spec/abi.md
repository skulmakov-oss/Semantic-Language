# PROMETHEUS ABI Specification

Status: draft v0
Owner crate: `prom-abi`

This document defines the narrow host ABI boundary between Semantic execution and PROMETHEUS-facing host services.

## Current ABI Surface

Current canonical host calls:

- `GateRead`
- `GateWrite`
- `PulseEmit`

Current owned planned post-stable call identities:

- `StateQuery`
- `StateUpdate`
- `EventPost`
- `ClockRead`

Current `v1` scope decision:

- the narrow ABI surface above is the official `v1` boundary
- the wider planned call family is not part of the current `v1` commitment

Explicit non-`v1` calls:

- `StateQuery`
- `StateUpdate`
- `EventPost`
- `ClockRead`

## Contract Rule

The ABI layer defines:

- host call identity
- effect classification
- determinism classification
- input/output envelope ownership
- host-side failure taxonomy

The ABI layer does not define:

- capability policy ownership
- verifier admission logic
- VM execution mechanics
- runtime orchestration

## Current Boundary

Current boundary owner:

- `prom-abi`

Current consumer:

- `sm-vm` through a narrow host trait

Current rule:

- host-effect opcodes in the VM must cross the `prom-abi` trait boundary rather than embedding ad hoc host logic
- declaring planned call identities in `prom-abi` does not by itself widen the
  current host trait or VM admission surface
