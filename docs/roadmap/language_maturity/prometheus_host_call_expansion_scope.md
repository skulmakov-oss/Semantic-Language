# PROMETHEUS Host-Call Expansion Scope

Status: completed post-stable first-wave expansion track
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

## Close-Out Reading

This track is now complete for the first-wave host-call family pack.

What is now admitted on `main`:

- `StateQuery`
- `StateUpdate`
- `EventPost`
- `ClockRead`

What remains unchanged:

- the published `v1.1.1` line still keeps the narrow `GateRead` / `GateWrite` /
  `PulseEmit` boundary as its official stable commitment
- `CapabilityManifest::gate_surface()` remains narrow `v1`-only
- no transport, persistence, replay, or richer runtime-effect expansion is
  implied by this completed track

The correct reading is therefore:

- first-wave post-stable host-call expansion is complete on `main`
- the published stable baseline is still narrower than current `main`
- any future widening beyond this pack is a new post-stable track, not a tail
  of the current one

## Planned First-Wave Families

Currently admitted post-stable families:

- `StateQuery`
- `StateUpdate`
- `EventPost`
- `ClockRead`

Remaining planned families:
- none in the current first-wave host-call expansion pack

The track does not imply that all planned families must land in one slice.

## Slice History

1. docs/governance checkpoint
2. ABI/capability declaration ownership for planned host-call families
3. `StateQuery` verifier/VM/runtime admission via generic host path
4. `StateUpdate` verifier/VM/runtime admission via generic host path
5. `EventPost` verifier/VM/runtime admission via generic host path
6. `ClockRead` verifier/VM/runtime admission via generic host path
