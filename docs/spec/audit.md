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
- `AuditReplayArchive`
- `MultiSessionReplayArchiveSession`
- `MultiSessionReplayArchive`
- `MultiSessionReplayArchiveFormatError`

## Ownership Rule

`prom-audit` owns:

- audit event schema
- centralized audit trail structure
- replay metadata schema
- replay archive envelope shape
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

Current persisted replay rule:

- `AuditReplayArchive` wraps session metadata, recorded events, and replay
  metadata under one explicit archive envelope
- archive metadata is explicit through `format_version`
- archive materialization/loading uses one canonical deterministic text envelope
- persisted replay ownership does not widen orchestration or runtime recovery
  semantics by implication

Current post-stable owner-layer widening on `main`:

- `prom-audit` now also owns explicit multi-session replay bundle types:
  - `MultiSessionReplayArchiveSession`
  - `MultiSessionReplayArchive`
- canonical text materialization/loading for multi-session replay is now
  admitted through one explicit deterministic envelope
- session bundles remain ordered by `session_ordinal`, which must be monotonic
  from zero
- this widening still does not imply rollback, recovery, or runtime replay
  orchestration

## Boundary Rule

Current architectural rule:

- `prom-runtime` may provide session metadata to initialize an audit trail
- `prom-audit` owns the shape of audit records and replay metadata
- future runtime hooks may emit into this schema, but must not redefine it
