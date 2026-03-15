# Standard Library Surface

Status: proposed v0

## Goal

Define the first intentional standard-library surface for Semantic beyond the
current math- and runtime-oriented builtins.

This workstream is about the public library contract that ordinary users can
import and rely on, not about widening the host ABI or hiding system effects
behind ad hoc helpers.

## Current Baseline

Today Semantic has:

- a deterministic compiler and VM pipeline
- a narrow executable source surface
- builtin math functions such as `sqrt`, `abs`, and `pow`
- runtime-oriented system layers around verification, quotas, gates, and
  PROMETHEUS boundary contracts

What it does not yet have is a broad user-facing standard library contract for
ordinary source programs.

Current honest constraints:

- no stable string library surface
- no stable collection library surface
- no stable filesystem or time modules as a source-language contract
- no canonical serialization or text-processing surface
- no clear public split between "language builtin" and "stdlib module"

## Why This Matters

A language cannot approach Rust or Python maturity without a usable standard
library.

Without a standard-library roadmap, users are forced to choose between:

- hard-coding logic into one-off builtins
- relying on unstable host-specific behavior
- avoiding common tasks entirely because there is no stable module story

## Design Principles

The standard library should preserve the current platform character.

Required principles:

- deterministic behavior by default
- explicit imports
- no hidden host effects
- compatibility with verifier-before-execution architecture
- clear separation between language primitives and library APIs
- staged expansion rather than one giant "stdlib" dump

## Builtin Versus Stdlib

The first discipline rule is that not everything should become a builtin.

Current intended split:

- builtins stay reserved for very small language-adjacent primitives
- stdlib modules handle reusable user-facing functionality
- host/ABI interactions remain explicit rather than disguised as harmless
  library sugar

The detailed boundary is specified in:

- `docs/roadmap/language_maturity/stdlib_boundary.md`

## Staged Expansion Plan

### Stage 1: Core Utility Families

The first stdlib wave should define a small number of high-value families:

- `math`
- `text`
- `collections`
- `serde`

Reason:

- these are the smallest families that make the language feel broader to users
- they stay close to deterministic computation
- they avoid opening system-effect scope too early

### Stage 2: Structured Runtime Utility Families

After the core utility wave, the next candidates are:

- `time`
- `path`
- `fs`

These families should land only with an explicit effect and capability story.

### Stage 3: Domain And Host-Oriented Families

Only after core utility and structured runtime modules should the repository
consider wider domain surfaces such as:

- policy helpers
- PROMETHEUS-facing helper modules
- richer semantic-state convenience APIs

These should not be mistaken for the first general-purpose stdlib wave.

## First Stable Families

The first intentional stdlib family map is defined in:

- `docs/roadmap/language_maturity/stdlib_module_families.md`

## Compatibility Rules

The stdlib cannot be treated as loose examples. Once a family is stabilized, it
becomes part of the public language surface.

Expected rules:

- module names must be stable once published
- exported function names must not silently drift
- capability-sensitive modules require explicit version review
- stdlib docs must distinguish proposed, experimental, and stabilized families

## Non-Goals

This workstream is not intended to:

- ship every possible library family in one wave
- hide runtime limits behind implicit host effects
- treat external integrations as if they were already stable stdlib
- turn host-bound system APIs into fake pure functions

## Acceptance Criteria

This workstream should be considered materially started only when:

- one standard-library roadmap exists with prioritized families
- language builtins and stdlib modules are clearly separated
- at least one library family has a documented public contract and examples
- library compatibility expectations are stated explicitly

## Immediate Next Slice

The immediate next slice for this PR is to define the first module-family map
and the builtin-vs-stdlib boundary, because that turns "stdlib" from a slogan
into an actual library plan.
