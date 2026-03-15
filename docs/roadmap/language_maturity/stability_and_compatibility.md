# Stability And Compatibility Discipline

Status: proposed

## Goal

Define the long-horizon compatibility and release discipline required for Semantic to be trusted as a full language platform.

## Why

Rust and Python earned maturity through years of stable releases, compatibility review, migration guidance, and user trust. Semantic still needs that durability story.

## Scope

- define stable-vs-draft-vs-experimental language surfaces
- define language-level compatibility review rules, not only bytecode-level rules
- define migration and deprecation expectations
- define release cadence and support-window expectations

## Non-Goals

- rewriting published history
- silently treating draft surfaces as stable
- turning every future feature into an immediate compatibility promise

## Acceptance Criteria

- one compatibility policy exists for source language, CLI, stdlib, and package surfaces
- deprecation and migration rules are explicit
- stable release criteria are described beyond a single tag event
- users can tell which parts of the language are stable and which are still evolving
