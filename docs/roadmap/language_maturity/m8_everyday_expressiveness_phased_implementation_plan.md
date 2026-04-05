# M8 Everyday Expressiveness Phased Implementation Plan

Status: active post-stable execution plan

Related documents:

- `docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`
- `docs/roadmap/language_maturity/m8_everyday_expressiveness_blueprint.md`

## Purpose

Translate the proposed M8+ package into milestone sequencing, PR-wave
discipline, and a narrow execution model.

## Milestone Table

| Milestone | Name | Focus | Status | Depends on |
|---|---|---|---|---|
| M8 | Everyday Expressiveness Foundation | text, package baseline, collections, closures | active | stable `v1.1.1` baseline |
| M9 | General Abstraction Layer | generics, traits, iterables, richer patterns | proposed | M8 core outputs |
| M10 | Application and Platform Expansion | UI boundary, concurrency, broader runtime/platform surfaces | proposed | M8 + selected M9 outputs |

## PR Wave Discipline

Default wave template:

- Wave 0: governance checkpoint and scope document
- Wave 1: owner-layer types and admitted surface inventory
- Wave 2: parser/sema/type-surface admission
- Wave 3: IR/runtime/VM or package-resolution execution path
- Wave 4: docs/tests/goldens/compatibility freeze

Default narrow PR pattern:

1. scope checkpoint or wave-opening docs update
2. owner-layer scaffolding
3. surface admission
4. execution/runtime/package path
5. freeze and close-out

## Phase Table

| Phase | Track | Goal | Output | Success Criteria |
|---|---|---|---|---|
| Phase 1 | M8.1 Text | introduce first-class text contract | text spec + implementation + tests | completed first-wave admitted type across parse/sema/IR/VM on current `main` |
| Phase 2 | M8.2 Packages | establish package/dependency contract | manifest/package baseline + docs + tests | package identity/dependency rules are explicit and reproducible |
| Phase 3 | M8.3 Collections | introduce minimum first-class collection carriers | collections spec + implementation + tests | at least one narrow collection baseline is usable and documented |
| Phase 4 | M8.4 Closures | introduce real closure values | closure spec + capture rules + runtime path | closures are no longer only immediate/pipeline sugar |
| Phase 5 | M9.1 Generics | open parametric abstraction | generics spec + implementation + tests | reusable typed abstractions become possible without surface ambiguity |
| Phase 6 | M9.2 Traits | open behavior abstraction | protocol/trait baseline + docs + tests | behavior contracts are expressible without ad hoc duplication |
| Phase 7 | M9.3/M9.4 | iterables + richer patterns | iterable/pattern docs + implementation | abstraction layer becomes practical rather than merely theoretical |
| Phase 8 | M10.1 | UI boundary | UI/application boundary contract | application/platform story becomes possible without contaminating the language roadmap |

## Track-by-Track Wave Reading

### M8.1 Text

- Wave 0: `text_type_full_scope.md`
- Wave 1: text type ownership and literal forms
- Wave 2: parser/sema/type admission
- Wave 3: IR/lowering/VM path
- Wave 4: docs/tests/compatibility freeze

Current completed checkpoint:

- `docs/roadmap/language_maturity/text_type_full_scope.md`

### M8.2 Package Ecosystem Baseline

- Wave 0: package scope checkpoint
- Wave 1: manifest/package identity ownership
- Wave 2: dependency declaration and module/package relationship admission
- Wave 3: resolution/lock baseline or explicit first-wave non-commitment
- Wave 4: docs/tests/compatibility freeze

Current completed checkpoint:

- `docs/roadmap/language_maturity/package_ecosystem_baseline_scope.md`

### M8.3 Collections

- Wave 0: collection scope checkpoint
- Wave 1: admitted collection family ownership
- Wave 2: construction/access/type-surface admission
- Wave 3: iteration/runtime/VM path
- Wave 4: docs/tests/compatibility freeze

### M8.4 Closures

- Wave 0: closure scope checkpoint
- Wave 1: closure value and capture-policy ownership
- Wave 2: source typing and invocation admission
- Wave 3: lowering/runtime representation
- Wave 4: docs/tests/compatibility freeze

## Primary Recommended Order

1. M8.1 Text / strings
2. M8.2 Package ecosystem baseline
3. M8.3 Collections
4. M8.4 First-class closures
5. M9.1 Generics
6. M9.2 Traits / protocols
7. M9.3 and M9.4 iterables / richer patterns
8. M10.1 UI application boundary

## Acceptable Alternative

If immediate data-shaping needs become more urgent than ecosystem scaling:

1. M8.1 Text / strings
2. M8.3 Collections
3. M8.2 Package ecosystem baseline
4. M8.4 First-class closures
5. M9.1 Generics
6. M9.2 Traits / protocols

## Operational Rule

Open a new track only when:

- it belongs to the current milestone order
- it has a scope checkpoint
- admitted contract is explicit
- exclusions are explicit
- it does not silently widen the published stable line
- it does not reopen a closed first-wave surface without a new versioned
  decision
