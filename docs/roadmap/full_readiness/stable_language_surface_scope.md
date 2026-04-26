# FR-1 Stable Language Surface Scope

Status: proposed readiness scope  
Parent: Semantic Full Readiness — Non-UI Track

## Goal

Freeze the minimum public source surface required before Semantic can be called a fuller practical language.

This scope does not implement features. It classifies the language surface into:

- promised / stable;
- qualified limited;
- landed on current main but not promised;
- explicitly out of scope.

## Required classification set

The following forms must be classified before this phase closes:

- function declarations and calls;
- `let` bindings;
- mutable locals;
- assignment/reassignment;
- `if` / `else`;
- `match`;
- loop forms;
- `break` / `continue`;
- `return`;
- block expressions;
- records;
- schemas;
- ADT/enum declarations and constructors;
- `Option` / `Result`;
- `Sequence`;
- imports and exports;
- function contracts: `requires`, `ensures`, `invariant`;
- core value types: `quad`, `bool`, `i32`, `u32`, `f64`, `fx`, `text`, `unit`.

## Required documents to audit

- `docs/spec/syntax.md`
- `docs/spec/types.md`
- `docs/spec/source_semantics.md`
- `docs/spec/modules.md`
- `docs/spec/diagnostics.md`
- README status section
- Wiki-facing current status document, if retained

## Issues / work packages

### FR-1.1 — freeze public syntax contour

Acceptance:

- syntax forms are listed in one matrix;
- each form has a status family;
- unsupported syntax is explicit.

### FR-1.2 — freeze source evaluation order

Acceptance:

- call argument order is documented;
- binary expression evaluation order is documented;
- match scrutinee evaluation is documented;
- pipeline stage order remains explicit.

### FR-1.3 — freeze type admission table

Acceptance:

- every public type has parse/sema/IR/SemCode/VM status;
- partial types are marked partial or current-main-only;
- no type is documented as stable if not executable through the relevant path.

### FR-1.4 — freeze match and ADT behavior

Acceptance:

- constructor, tag, payload, and match behavior are classified;
- exhaustiveness status is explicit;
- unsupported pattern forms are listed.

### FR-1.5 — freeze imports/modules behavior

Acceptance:

- direct import, selected import, alias, wildcard, re-export status is explicit;
- deterministic resolution order is documented;
- known out-of-scope module/package behavior is listed.

### FR-1.6 — document unsupported syntax explicitly

Acceptance:

- unsupported everyday forms produce deterministic diagnostics where possible;
- user-facing docs do not imply accidental support;
- examples avoid unsupported forms unless intentionally negative.

## Out of scope

- UI language boundary;
- broad generics/traits;
- macro system;
- package registry;
- implementation of missing features;
- stable release promotion.

## Definition of Done

FR-1 is complete when the public source surface has a single classification matrix and no release-facing document claims support that is absent, partial, or out of scope.
