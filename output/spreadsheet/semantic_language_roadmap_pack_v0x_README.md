# Semantic Language Roadmap Pack (`v0.1` / `v0.2` / `v0.3`)

This roadmap pack turns the earlier pyramid into a concrete import-friendly
backlog:

- `v0.1` = density surface
- `v0.2` = contract and data core
- `v0.3` = schema and boundary core

Files:

- `output/spreadsheet/semantic_language_roadmap_pack_v0x.csv`

Columns:

- `Release`: planned release wave (`v0.1`, `v0.2`, `v0.3`)
- `WBS ID`: stable roadmap identifier
- `Merge Order`: strict recommended implementation order
- `Issue`: canonical work item name
- `Priority`: relative priority inside the release wave
- `Owner Crate`: lead owner based on the current module ownership map
- `Touch Crates`: expected implementation surface
- `Depends On`: roadmap prerequisites by `WBS ID`
- `Acceptance Criteria`: one-line definition of done

## Architectural intent

This pack is deliberately staged by cost:

1. `v0.1` stays mostly in `sm-front`, `sm-profile`, `sm-sema`, and `sm-ir`.
2. `v0.2` is the first real value-model wave, so it starts touching
   `sm-runtime-core`, `sm-emit`, and `sm-vm`.
3. `v0.3` adds schema and generated-contract value without reopening the
   `PROMETHEUS` integration boundary.

## Scope guard

- `v0.1` should not touch `prom-*`.
- `v0.2` may expand the core value model, but still should not reopen
  `prom-abi`, `prom-cap`, `prom-state`, `prom-rules`, `prom-runtime`, or
  `prom-audit`.
- `v0.3` introduces schema/config/API contract generation, but still remains a
  `Semantic Core + tooling` wave rather than a runtime-integration wave.

## Recommended execution order

### `v0.1`

1. `V01-01` Expression-valued control core
2. `V01-02` Guarded control core
3. `V01-03` Composition surface
4. `V01-04` Lambda and wildcard pattern surface
5. `V01-05` Flow density primitives
6. `V01-06` Call-site density
7. `V01-07` Contract visibility basics
8. `V01-08` Const and literal density

### `v0.2`

9. `V02-01` Tuple value model
10. `V02-02` Tuple destructuring and let-else
11. `V02-03` Record value model and immutable update
12. `V02-04` ADT and sum-type core
13. `V02-05` Exhaustive match enforcement
14. `V02-06` Result and Option first-class ergonomics
15. `V02-07` Units of measure
16. `V02-08` Function contracts and invariants
17. `V02-09` Default parameter semantics

### `v0.3`

18. `V03-01` Schema-first declarations
19. `V03-02` Validation derived from schemas and types
20. `V03-03` Config schema contract
21. `V03-04` Generated API contract surface
22. `V03-05` Versioned schemas and migrations
23. `V03-06` Tagged wire unions and patch types

## How to use

1. Import each row as one GitHub issue.
2. Use `Release` as the milestone grouping.
3. Preserve `Merge Order`; do not merge the wave in random order.
4. Treat `Depends On` as real prerequisites, not just hints.
5. Keep desugaring notes, diagnostics, formatter behavior, and verified-path
   tests mandatory for every feature wave.
