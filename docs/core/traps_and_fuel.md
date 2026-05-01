# Traps And Fuel

## Trap model

Runtime uses the following stable trap set:

- `InvalidPc`
- `InvalidRegister`
- `TypeMismatch`
- `DivisionByZero`
- `IntegerOverflow`
- `FuelExceeded`
- `CallDepthExceeded`
- `InvalidFunction`
- `AssertFailed`
- `ExplicitTrap`

`Ret` from the entry frame is the normal completion path, so the public trap set intentionally does not include a stack-underflow case.

## Fuel model

- Fuel is a decreasing counter.
- One executed instruction consumes one fuel unit.
- Execution stops with `FuelExceeded` when the next instruction cannot be funded.
- `fuel_used` is part of the deterministic result digest.
