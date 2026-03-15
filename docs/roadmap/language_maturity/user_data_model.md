# User Data Model Expansion

Status: proposed

## Goal

Expand Semantic from a narrow reasoning core into a language with a broader user-facing data model.

## Why

Rust and Python are not only execution pipelines. They also expose a rich model of values and user-defined abstractions. Semantic remains narrow by comparison.

## Scope

- define the canonical user-facing aggregate type story
- define whether `struct`-like records, tuples, and collections are part of the language
- define construction, access, equality, and pattern semantics for user data
- define how quad-oriented reasoning composes with richer user data

## Non-Goals

- replacing the current deterministic execution model
- turning post-`v1` runtime features into immediate blockers
- importing Python-style dynamic semantics by default

## Acceptance Criteria

- the repository has a documented canonical data-model roadmap
- at least one intentional user-defined aggregate family is specified end-to-end
- examples demonstrate real user data beyond scalar and quad-only flows
- type and lowering rules for the new data model are documented rather than inferred from implementation accidents
