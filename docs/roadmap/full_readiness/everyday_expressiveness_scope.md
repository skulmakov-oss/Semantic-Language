# FR-2 Everyday Expressiveness Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Close the ordinary programming surface needed before Semantic can be treated as a practical language rather than only a verifier-first execution platform.

This scope defines what must be implemented or explicitly classified. It does not itself implement features.

## Required capability families

- public integer arithmetic;
- mutable locals;
- reassignment;
- `while`;
- statement `loop`;
- `break` and `continue`;
- block expression consistency;
- text operations sufficient for small programs;
- collection use sufficient for stateful algorithms;
- closure usability hardening;
- basic `Option` / `Result` flow;
- deterministic diagnostics for unsupported everyday forms.

## Work packages

### FR-2.1 — integer arithmetic public surface

Scope:

- classify supported numeric families;
- define overflow behavior or explicit non-commitment;
- ensure source/IR/SemCode/VM path is documented for admitted operations.

Acceptance:

- ordinary `i32` arithmetic status is explicit;
- numeric errors/traps are deterministic;
- tests exist for admitted arithmetic behavior once implemented.

### FR-2.2 — mutable locals and reassignment

Scope:

- define source syntax;
- define typing restrictions;
- define runtime/lowering effects;
- prevent accidental hidden mutation.

Acceptance:

- mutation is explicit;
- immutable binding reassignment is rejected;
- mutable reassignment lowers deterministically.

### FR-2.3 — while loop

Implementation scope:

- `docs/roadmap/full_readiness/while_statement_scope.md`

Scope:

- define condition type requirements;
- define lowering shape;
- define quota interaction;
- define break/continue interaction where applicable.

Acceptance:

- `while` requires boolean condition;
- infinite loops are bounded by runtime quotas;
- lowering is deterministic.

### FR-2.4 — statement loop and control exits

Scope:

- define statement loop syntax;
- define `break` and `continue` diagnostics;
- define nested loop behavior;
- define whether loop expressions with values are admitted in this track.

Acceptance:

- control exits are only valid in loop context;
- nested behavior is deterministic;
- unsupported value-carrying loop cases are explicit.

### FR-2.5 — block expression consistency

Scope:

- classify block expression forms;
- define tail expression behavior;
- define interaction with `return`, `break`, and contracts.

Acceptance:

- source semantics document has one rule set;
- inconsistent tail-return behavior is eliminated or explicitly rejected.

### FR-2.6 — text/string usability

Scope:

- minimum concatenation or formatting path;
- conversion to text for core values;
- diagnostics for unsupported formatting.

Acceptance:

- small programs can produce readable text output once IO/debug surface exists;
- text behavior does not bypass runtime/value contracts.

### FR-2.7 — closure usability hardening

Scope:

- freeze capture policy;
- classify mutable capture status;
- define invocation and typing behavior;
- keep closure runtime representation deterministic.

Acceptance:

- closure behavior is clear enough for examples;
- unsupported capture modes fail deterministically.

### FR-2.8 — diagnostics for unsupported everyday patterns

Scope:

- identify common forms users will try;
- add or document clear diagnostics;
- avoid silent parse ambiguity.

Acceptance:

- unsupported everyday syntax produces actionable feedback;
- negative examples are stable.

## Out of scope

- UI applications;
- broad generics/traits;
- macro system;
- async/concurrency;
- package registry;
- host/runtime expansion outside explicit capability scope.

## Definition of Done

FR-2 is complete when a small stateful program can be written without workaround-heavy code, and every admitted everyday construct has deterministic source, lowering, verification, and runtime behavior where applicable.
