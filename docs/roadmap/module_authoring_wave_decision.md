# Module Authoring Wave Decision

Status: completed `B0` decision note
Primary owners: language maturity, practical-programming readiness

## Purpose

This note records the evidence-driven choice for the single `B2` widening wave
in the readiness-completion plan.

It does not widen the executable module contour by itself.

## Evidence Base

Draft canonical examples used for this note:

- `examples/readiness_draft_canonical/cli_batch_core`
- `examples/readiness_draft_canonical/rule_state_decision`
- `examples/readiness_draft_canonical/data_audit_record_iterable`
- `examples/readiness_draft_canonical/wave2_local_helper_import`
- `examples/readiness_draft_canonical/module_selected_import_settlement`
- `examples/readiness_draft_canonical/module_selected_import_audit_report`

Executable module contour at decision time:

- direct local-path bare helper-module imports
- direct helper declarations bundled into the executable semantic path
- no selected import
- no alias import
- no wildcard import
- no namespace-qualified executable access

## Example Friction Reading

### 1. `cli_batch_core`

Module pressure:

- none

Reading:

- this example validates the single-file executable core but does not influence
  the next module wave choice

### 2. `rule_state_decision`

Module pressure:

- none

Reading:

- this example validates the natural rule/state shape but does not influence
  the next module wave choice

### 3. `data_audit_record_iterable`

Module pressure:

- none

Reading:

- this example validates the iterable/data contour but does not influence the
  next module wave choice

### 4. `wave2_local_helper_import`

Module pressure:

- low, but still narrow

Current benefit:

- proves that one helper module can already be admitted end to end

Current limitation:

- every imported executable declaration arrives as an unqualified root binding
- there is no way to request only the symbols the root file actually uses

### 5. `module_selected_import_settlement`

Module pressure:

- high

Observed friction:

- two helper modules both export `status_text`
- the natural source shape wants symbol-level import plus aliasing
- the current bare-import-only contour would force unnatural helper renaming or
  helper-file reshaping just to avoid executable root-scope collision

### 6. `module_selected_import_audit_report`

Module pressure:

- high

Observed friction:

- the root file wants only four helper functions
- the current bare-import-only contour would import every helper declaration
  into the executable root scope
- this creates avoidable scope spillover and makes later helper growth more
  collision-prone than the example logic warrants

## Chosen `B2` Wave

Chosen wave:

- `selected import`

Post-decision reading:

- this note records why `selected import` was chosen at the time
- it is not the authority for the current admitted executable contour after the
  later `B2` implementation and `C1` Gate 1.1 re-synthesis work
- current factual release-facing reading now lives in:
  - `docs/roadmap/v1_readiness.md`
  - `reports/g1_release_scope_statement.md`
  - `docs/roadmap/language_maturity/executable_module_entry_scope.md`

## Why `selected import` Wins

1. It removes the strongest friction shown by the draft examples:
   - symbol-level curation
   - aliasing for same-named helper exports
   - avoiding root-scope spillover from helper-heavy modules

2. It directly addresses the already-preserved blocked probe from the first
   qualification cycle:
   - `examples/qualification/g1_real_program_trial/module_helpers_blocked`

3. It is the tighter widening:
   - the broader module contract already owns selected import syntax
   - executable-path admission is the missing piece

4. It removes a more structural pain point than namespace-qualified access:
   - namespace-qualified access would improve provenance, but it would still
     leave import-all behavior and symbol-selection pain unresolved

## Why `namespace-qualified executable access` Is Not First

`namespace-qualified executable access` remains a valid later candidate, but it
is not the highest-value first step for readiness completion.

Reason:

- the draft examples do not primarily fail because they need repeated namespace
  calls
- they fail because they need symbol-level import control and aliasing
- selected import therefore removes the higher-value blocker first

## Resulting `B2` Boundary

`B2` should admit exactly one narrow widening wave:

- selected import on the executable path

Still out of scope for `B2`:

- wildcard import admission
- public re-export admission on the executable path
- package-qualified executable widening beyond the already landed baseline
- namespace-qualified executable access
- any generalized import/package redesign
