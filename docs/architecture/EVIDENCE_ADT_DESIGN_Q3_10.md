# Q3.10 design only — Opaque Evidence ADT and Authenticated Proof Authority

**Status:** PROPOSAL / NOT IMPLEMENTED. **No change authorized to frozen C0, SemCode ABI, VM or verifier.** This document describes the language/runtime changes needed beyond Q3.9/Q4.9.

## Problem

Semantic's current `record Evidence` and `record TrustedSemAdmission` are ordinary public records. `with` expressions can fabricate state, witness sets or authorization flags. Checking their shape and replaying Sema inside `compile_source` protects that driver, but cannot prove that an arbitrary caller holds an authentic proof object. Marking a struct field `private` in documentation alone does not close construction.

## Proposed trust contract

Define an opaque `VerifiedEvidence` value **whose representation cannot be constructed, modified, pattern-extracted, serialized into a trusted form or reinterpreted by untrusted Semantic code**. Its issuer is the authorized Verifier/Compiler Evidence Service, not a source-level public constructor.

A proof issue request should bind the following separately and precisely:

1. **Proposition identity:** proposition kind, owner AST ID, semantic scope and type obligation ID; JOIN only when all identity coordinates are identical.
2. **Witness identity:** ordered positive/negative observations each referencing an authenticated `SourceOrigin(file_id, offset, source_set_id)` and the derivation rule that produced them.
3. **Authority context:** canonical source-set content digest, parser/typing specification version, compiler implementation identity, epoch/policy namespace and allowed operation.
4. **Derivation:** parent proof handles, admission rule identifier, deterministic order, optional optimization transfer proofs, plus the final specific lowering precondition being authorized.

`N/F/T/S` describe the logical evidence state and are computed from witnesses; no caller may supply a trusted state byte. `S` requires both opposing observations **of the same proposition**; unrelated keys/values or condition/branch claims can never join.

## Minimal required implementation work

- Language surface: module/visibility boundaries or a dedicated opaque primitive whose construction and `with` updates are statically rejected outside the issuer. Treat `Evidence` raw data as untrusted input and `VerifiedEvidence` as an unforgeable type.
- Verifier/runtime: validate or mint capability-tagged proof handles on admission, enforce constructor/field/serialization restrictions and trust separation even for hand-crafted SemCode. Avoid a security model that depends only on the frontend parser.
- Source identity: replace Q4.9's path+raw-text snapshot with an externally reproducible canonical multi-file byte manifest and a collision-resistant hash; bind path canonicalization and import graph. Preserve the exact file-qualified origin in issued witness handles.
- Lowering/optimization: require explicit valid proof handles per typed obligation and a deterministic chain of transfer rules for each transformation; reject stale epoch/unknown rule/invalid parent identities.
- Versioned ABI: define opt-in proof transport in a separately governed SemCode revision; frozen C0 and older artifacts remain unchanged until an owner-approved migration.

## Required red-team tests

Reject manual `VerifiedEvidence { ... }` construction; `with` mutation; fabricated T/S; cross-proposition JOIN; duplicate or swapped witness origins; cross-file same-offset substitution; replay from another source-set digest/epoch; changed AST after seal; forged raw SemCode proof section; version downgrade and unsupported proof rules. Preserve deterministic output across repeat builds and cross-platform paths.

## Architecture decisions pending explicit owner GO

Opaque representation: compiler-enforced nominal private type versus verifier-minted handle/capability. Transport: non-emitted proof sidecar versus versioned SemCode proof section. Authentic source identity: reproducible source manifest specification and digest algorithm. Threat model: accidental inconsistency, hostile Semantic source, hostile plug-ins, direct forged SemCode, or hostile filesystem. Compatibility: no silent change to frozen bootstrap interfaces.

**Definition of Done for actual Q3.10:** failure to construct or mutate a trusted carrier from untrusted Semantic and from raw SemCode; end-to-end verifier enforcement; source-bound handle replay rejection; rigorous positive/negative tests; owner-authorized contract and a qualified C1/C2 bootstrap chain. None is claimed today.
