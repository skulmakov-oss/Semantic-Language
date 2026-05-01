# Backend Policy

The public execution contract is defined by the scalar backend.

## Policy

- `Scalar` is the reference backend.
- `Auto` may select a faster backend later, but every public result must match scalar behavior.
- Capability reporting is limited to standard CPU feature flags such as `popcnt`, `bmi1`, `bmi2`, `avx2`, `avx512`, `neon`, and `sve`.
- Backend internals are not part of the public API surface.
