# FR-5 / FR-6 Execution and Runtime Closure Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Close verifier-first execution and deterministic runtime behavior as public readiness contracts.

This document scopes execution/runtime closure. It does not implement verifier, VM, or runtime changes.

## FR-5 — Verified Execution Closure

### Purpose

Ensure the standard execution path remains verifier-first and that malformed or unauthorized SemCode is rejected before VM execution.

### Work packages

#### FR-5.1 — freeze SemCode admission matrix

Acceptance:

- opcode families are mapped to required SemCode versions;
- capability-gated operations have explicit admission rules;
- unsupported opcodes reject deterministically.

#### FR-5.2 — freeze capability manifest validation

Acceptance:

- manifest structure is documented;
- required capability bits are checked before execution;
- missing capability behavior is deterministic.

#### FR-5.3 — freeze section integrity checks

Acceptance:

- section ordering/bounds rules are explicit;
- corrupt sections reject before VM execution;
- rejection diagnostics remain stable.

#### FR-5.4 — freeze control-flow validation

Acceptance:

- jump targets are validated;
- call discipline is validated;
- register bounds are validated;
- invalid programs do not reach execution.

#### FR-5.5 — preserve verified-only default run path

Acceptance:

- raw execution remains internal/test-only if retained;
- public CLI execution uses verified program envelope;
- docs do not imply VM is the primary admission boundary.

#### FR-5.6 — malformed SemCode golden plan

Acceptance:

- malformed header cases;
- invalid opcode cases;
- invalid section cases;
- invalid capability cases;
- invalid control-flow cases.

## FR-6 — Deterministic Runtime Closure

### Purpose

Ensure runtime behavior is reproducible, bounded, and diagnosable.

### Determinism invariant

```text
same source
+ same compiler config
+ same SemCode
+ same runtime config
+ same capability manifest
+ same input/event stream
= same result / same trap / same trace class
```

### Work packages

#### FR-6.1 — freeze RuntimeValue set

Acceptance:

- all public runtime value families are listed;
- unsupported source values do not create hidden runtime variants;
- value display/debug behavior is explicit where relevant.

#### FR-6.2 — freeze SymbolId runtime model

Acceptance:

- runtime identity is SymbolId-based;
- debug names remain diagnostic metadata;
- no hot path depends on user-facing strings as identity.

#### FR-6.3 — freeze quota/fuel taxonomy

Acceptance:

- max steps/calls/frames/effect calls/trace entries are classified;
- quota-exhaustion traps are documented;
- CLI/config hooks are explicit or deferred.

#### FR-6.4 — freeze trap taxonomy

Acceptance:

- deterministic traps are listed;
- trap payload stability is defined;
- unsupported runtime cases fail predictably.

#### FR-6.5 — freeze trace/audit event shape for core execution

Acceptance:

- core trace classes are stable enough for release qualification;
- host/effect trace events are separated from pure VM trace;
- trace output does not become a hidden semantic contract unless explicitly promised.

#### FR-6.6 — deterministic rerun tests plan

Acceptance:

- repeated execution tests are defined for pure programs;
- quota exhaustion tests are defined;
- capability-denial tests are defined;
- ownership-overlap rejection remains deterministic.

## Out of scope

- new host-call families;
- UI capability;
- runtime ownership expansion beyond current admitted slice;
- broad async/concurrency;
- rule-engine semantic widening;
- release promotion.

## Definition of Done

FR-5/FR-6 are complete when standard execution is verifier-admitted by default, malformed programs are rejected before VM execution, runtime quotas/traps are deterministic, and repeated runs with the same inputs produce the same result, trap, or trace class.
