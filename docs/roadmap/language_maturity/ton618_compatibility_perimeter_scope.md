# TON618 Compatibility Perimeter Scope

Status: proposed post-stable closure track
Related backlog item: `keep the explicit ton618_core / ton618-core compatibility perimeter narrow and documented`

## Goal

Keep the remaining `ton618_core` / `ton618-core` compatibility names explicit,
non-owning, and mechanically guarded without reopening root CLI scope or
reintroducing legacy ownership drift.

This is a post-stable perimeter-hardening track, not a new feature wave.

## Why This Exists

The current stable baseline already keeps the legacy TON618 names on a narrow
compatibility path:

- `src/bin/ton618_core.rs` remains only as a retained legacy CLI shim
- `crates/ton618-core` remains only as a retained compatibility-named primitive
  crate
- `ton618_legacy/` remains only as a retained historical source archive for the
  pre-`sm-*` naming era
- canonical public ownership for CLI, frontend, IR, SemCode, VM, and profile
  contracts already lives in the `sm-*` owners

The remaining work is therefore not to expand TON618 behavior. It is to freeze
the perimeter so the retained names cannot silently become second owners later.

## Included In This Track

- inventory of all remaining `ton618_core` / `ton618-core` compatibility entry
  points and re-export surfaces
- explicit guard coverage for the allowed perimeter
- docs sync across root/architecture/release-facing pages so the retained names
  are described as compatibility shims only
- narrow cleanup only where it reduces duplicated compatibility wording or
  helper drift without changing user-visible stable behavior

## Explicit Non-Goals

- redesigning the public CLI
- widening `ton618_core` command surface
- moving canonical ownership back from `sm-*` crates into TON618-named paths
- changing runtime, verifier, SemCode, or PROMETHEUS boundaries
- branding/naming rewrite of published release artifacts

## Intended Slice Order

1. docs/governance checkpoint
2. exact inventory + allowlist freeze for the remaining TON618-named perimeter
3. narrow cleanup of duplicated compatibility helpers or wording
4. docs-only close-out

## Acceptance Reading

This track is done only when:

- every remaining TON618-named path is explicitly justified as compatibility
  perimeter
- guard tests hold the allowed perimeter mechanically
- architecture and legacy docs all agree that TON618 names are non-owning shims
- no part of the track widens stable CLI or runtime behavior

## Slice History

1. docs/governance checkpoint
2. exact path/content inventory freeze for the remaining TON618-named perimeter
