# Quad Algebra

The quad domain uses the frozen encoding below:

| State | Bits | True plane | False plane |
| --- | --- | --- | --- |
| `N` | `00` | 0 | 0 |
| `F` | `01` | 0 | 1 |
| `T` | `10` | 1 | 0 |
| `S` | `11` | 1 | 1 |

## Core operations

- `join(a, b)` is bitwise OR on the two-bit encoding.
- `meet(a, b)` is bitwise AND on the two-bit encoding.
- `inverse(a)` swaps the true and false planes.
- `QImpl(a, b)` is `join(inverse(a), b)`.

## Packed forms

- `QuadroReg32` stores 32 quad lanes in one `u64`.
- `QuadTile128` stores 128 quad lanes as dual `u128` planes.
- `QuadMask32` and `QuadMask128` expose typed lane masks for projection and update.
