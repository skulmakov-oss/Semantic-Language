# PROMETHEUS ABI Specification

Status: draft v0
Owner crate: `prom-abi`

This document defines the narrow host ABI boundary between Semantic execution and PROMETHEUS-facing host services.

## Current ABI Surface

Current canonical host calls:

- `GateRead`
- `GateWrite`
- `PulseEmit`

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
