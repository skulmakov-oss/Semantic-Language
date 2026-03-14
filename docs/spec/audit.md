# PROMETHEUS Audit Specification

Status: draft v0
Owner crate: `prom-audit`

This document defines the current canonical audit and replay metadata contract for Semantic runtime execution.

## Current Audit Surface

Current canonical audit types:

- `AuditSessionMetadata`
- `AuditEventId`
- `AuditEventKind`
- `AuditEvent`
- `AuditTrail`
- `ReplayMetadata`

## Ownership Rule

`prom-audit` owns:

- audit event schema
- centralized audit trail structure
- replay metadata schema
- capability denial and host-effect event representation

`prom-audit` does not own:

- execution session orchestration
- state storage invariants
- rule scheduling semantics
- ABI descriptor semantics

## Event Rule

Current event families:

- session start and finish
- capability denial
- gate read and gate write
- pulse emit
- rule activation
- state transition
- free-form notes

Current event invariant:

- audit event ids are assigned monotonically within one audit trail

## Replay Rule

Current replay metadata must include:

- execution context
- capability manifest metadata
- whether a gate registry was bound
- event count
- last event id

## Boundary Rule

Current architectural rule:

- `prom-runtime` may provide session metadata to initialize an audit trail
- `prom-audit` owns the shape of audit records and replay metadata
- future runtime hooks may emit into this schema, but must not redefine it
