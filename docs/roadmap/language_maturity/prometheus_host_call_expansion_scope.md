# PROMETHEUS Host-Call Expansion Scope

Status: proposed post-stable expansion track
Related backlog item:
`wider PROMETHEUS host-call families beyond the narrow v1 boundary`

## Goal

Extend the current narrow PROMETHEUS host boundary beyond `GateRead`,
`GateWrite`, and `PulseEmit` in a staged way that keeps ABI, capability,
verifier, VM, runtime, and diagnostics ownership explicit.

This is a post-stable boundary-expansion track, not a correction to the
published `v1.1.1` line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- canonical host calls are limited to `GateRead`, `GateWrite`, and `PulseEmit`
- capability mapping is limited to the same narrow family
- VM host-effect execution crosses the `prom-abi` boundary through the current
  narrow trait surface only
- runtime/audit tests and compatibility notes cover only the narrow host-call
  family above

That baseline remains the source of truth until this track explicitly lands a
widened contract.

## Included In This Track

- explicit ownership of additional host-call families at the ABI layer
- matching capability taxonomy and denial-path ownership for the admitted calls
- verifier/VM/runtime admission for the same widened host-call surface
- diagnostics/spec/tests/goldens sync for the widened boundary contract

## Explicit Non-Goals

- changing the existing semantics of `GateRead`, `GateWrite`, or `PulseEmit`
- widening unrelated language/runtime surfaces by implication
- reopening `v1.1.1` docs as if wider PROMETHEUS calls already shipped there
- transport, persistence, replay, or rule-side effect expansion beyond the
  host-call family itself
- package, CLI, or release-layout redesign

## Intended Slice Order

1. docs/governance checkpoint
2. ABI/capability declaration ownership for planned calls
3. verifier/VM/runtime admission for one narrow additional host-call family
4. diagnostics/spec/runtime matrix freeze for the widened boundary
5. repeat per additional host-call family instead of widening all at once

## Acceptance Reading

This track is done only when:

- every admitted host call has explicit ABI identity and capability ownership
- verifier, VM, runtime, and audit behavior agree on the same widened surface
- diagnostics and runtime-matrix coverage make the widened contract inspectable
- stable-facing docs continue to distinguish the published `v1.1.1` boundary
  from the post-stable widened contract

## Planned First-Wave Candidates

The current non-`v1` candidate family remains:

- `StateQuery`
- `StateUpdate`
- `EventPost`
- `ClockRead`

The track does not imply that all four must land in one slice.

## Slice History

1. docs/governance checkpoint
2. ABI/capability declaration ownership for planned host-call families
3. `StateQuery` verifier/VM/runtime admission via generic host path
