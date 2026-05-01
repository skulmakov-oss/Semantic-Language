# Execution Core

The execution core is a deterministic register machine for the public Semantic runtime.

## Guarantees

- Each instruction consumes one unit of fuel.
- Runtime traps are reported through stable trap codes.
- Scalar execution is the reference behavior for every public result.
- Program validation checks entry points, register bounds, call targets, and jump targets before execution.
- Result digests are computed from status, trap code, return value, and fuel used only.
- The public `.core.json` lab and golden envelope is versioned through a required `format_version` field.

## Shape

- `QuadState` is the base four-state value domain.
- `CoreValue` carries the public primitive execution values.
- `Instr` is the typed instruction form used by the capsule.
- `CoreExecutor` runs validated programs with bounded fuel and call depth.
- `Ret` in the entry frame completes execution and returns the final value directly.
