## Bootstrap unit

<!-- Name the exact bootstrap/migration unit. -->

## Reference contract

- Reference repository: `skulmakov-oss/Semantic`
- Reference commit/ref:
- Current contract owner/module:
- Related upstream issue(s):

## Purpose

<!-- What exact behavior or contract is mirrored, qualified, or migrated? -->

## Migration state

- [ ] REFERENCE_ONLY → MIRRORED
- [ ] MIRRORED → DIFFERENTIAL
- [ ] DIFFERENTIAL → QUALIFIED
- [ ] QUALIFIED → CANONICAL
- [ ] Qualification/maintenance only; no state transition

## Observable equivalence rule

<!-- State exactly what is compared: bytes, tokens, normalized AST/IR, diagnostics, runtime result/trace, etc. -->

## Evidence

- [ ] Positive/reference cases
- [ ] Negative/error cases where contractual
- [ ] Boundary cases
- [ ] Differential comparison
- [ ] Mutation/adversarial proof that the gate detects a meaningful defect
- [ ] No unexplained deltas

## Scope

### In scope

-

### Explicitly out of scope

-

## Upstream drift check

- [ ] No known upstream issue invalidates this exact contract
- [ ] Reference point is recorded
- [ ] Any intentional divergence has an explicit contract decision

## Architecture checks

- [ ] No silent language redesign
- [ ] No dependency inversion introduced for convenience
- [ ] No UI dependency added to the bootstrap-critical path
- [ ] Rust reference/oracle path is not removed prematurely

## Qualification result

<!-- Commands, corpus size, exact comparison result, and remaining limitations. -->
