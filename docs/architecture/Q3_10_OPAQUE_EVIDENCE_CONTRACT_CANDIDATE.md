# Q3.10-A — Opaque VerifiedEvidence Contract Candidate

**Status:** DESIGN CANDIDATE / OWNER DECISION REQUIRED / NOT IMPLEMENTED
**Date:** 2026-10-31
**Issue:** #47
**Predecessor:** Draft PR #46 at 759ae2f5d327f2aac82f004cead552eb8be49229, stacked on Draft PR #45 at 972066b25d2201344747becff0a0d6abfeb0bf4d
**Canonical design starting point:** docs/architecture/EVIDENCE_ADT_DESIGN_Q3_10.md
**Authority:** Semantic-Language AGENTS.md, active Harness, upstream skulmakov-oss/Semantic#1910.

This is a proposal for owner review, not an accepted language contract or proof of security. In particular, adding private-looking source declarations would **not** by itself deliver Q3.10.

## 1. Problem statement and currently supported guarantees

- In compiler/sema.sm, Evidence is a public record with proposition_kind, subject, scope, state, positive and negative. Source code currently constructs and modifies it through record literals and with expressions.
- In compiler/lower.sm, TrustedSemAdmission is a public record. q3_trusted_admit reconstructs Sema results from the supplied parsed program and checks the AST-indexed evidence and type metadata. This protects the canonical compiler driver, **not** an arbitrary caller holding a record.
- Q4.9 in compiler/driver.sm checks source snapshots, AST reparse identity, source→AST→evidence→FnIr links, and refuses invalid links before emission. This is in-process provenance; it is not a sealed proof, a canonical digest or a verifier-visible attestation.
- Frozen C0, its verifier, the current SemCode ABI and the current Harness must remain untouched within this candidate.

**Target invariant:** No untrusted Semantic source program or hand-authored SemCode can manufacture an admitted proof by supplying a convincing record, field, integer, raw byte string or deserialized value. Evidence is admitted only when its origin, proposition, authorized rule and execution context have been independently validated.

## 2. Threat model / non-goals

In scope:
1. Untrusted Semantic source using constructors, with, field access, pattern extraction, casts and helper calls.
2. Hand-authored, malformed, downgraded or modified SemCode, including proof sections.
3. Replayed proof data from another source-set, compilation epoch or policy namespace.
4. Cross-proposition, cross-file, cross-AST and cross-obligation evidence substitution.
5. Unsupported transfer rules or parent proof IDs.
6. Deterministic identity and version mismatch across builds and target platforms.

Not guaranteed by the initial version: protection against a compromised verifier/VM, malicious process memory rewriting, malicious kernel, forged trusted loader, compromised cryptographic implementation, or an adversarial race in a mutable filesystem. Digest binding identifies bytes; **it does not establish who authored them**. A check-then-reopen source file still has a TOCTOU race unless build inputs are sealed as immutable bytes.

## 3. Decision matrix (candidates, not approvals)

| Decision | Option A | Option B | Recommended direction |
| --- | --- | --- | --- |
| Public representation | nominal private type only | verifier-issued opaque typed handle | B, with source-level nominal typing as an additional barrier |
| Issuer | Semantic function returning a record | trusted Verifier / Compiler Evidence Service | trusted issuer, not public constructor |
| Validation | parser/typechecker only | admission + verifier + VM type isolation | second path is mandatory for hostile SemCode |
| Proof transport | non-emitted internal sidecar | versioned verifier-visible proof/admission envelope | sidecar for exploratory tests only; governed verifier-visible envelope for actual guarantee |
| Source identity | loader path and exact text | canonical multi-file byte manifest with digest | B, after portable path/import rules are frozen |
| Bootstrap compatibility | silently repurpose existing header | opt-in revision with explicit rejection of unknown versions | B |

A bare integer handle, UUID, signature-looking text, public seal field or hash of self-declared evidence is **not** an unforgeable proof. The property comes from the issuer's ownership of the handle table, verifier validation of the proof graph, typed VM isolation, and refusal to deserialize arbitrary bytes as an already-admitted handle.

## 4. Proposed logical types (schema only; NOT valid Semantic syntax)

~~~text
SourceSetId     = DomainSeparatedSHA256(canonical_manifest_bytes)
PropositionId   = (kind, ast_owner_id, semantic_scope_id, obligation_id)
SourceOrigin    = (source_set_id, file_id, exact_byte_offset)
Witness         = (polarity, source_origin, derivation_rule_id, parent_proof_ids)
ProofContext    = (source_set_id, parser_contract_version,
                   type_contract_version, compiler_identity,
                   policy_namespace, epoch, allowed_operation)
ProofClaim      = (proposition_id, ordered_witnesses,
                   rule_id, parents, context)
VerifiedEvidence = opaque capability-typed handle issued after validation
~~~

The tuples above are **semantic design fields**; they do not define a new encoded wire format.

- The trusted evidence state N/F/T/S is derived from distinct authenticated positive and negative witnesses.
- S means opposing evidence for the **same exact** PropositionId. Two independent propositions cannot yield S by JOIN.
- JOIN is permitted only for identical PropositionId AND compatible ProofContext (including source set, policy, epoch and operation).
- An operation requiring a positive T proof must reject N, F and S. Conflict S is preserved, not silently collapsed.
- Witness order and deduplication are deterministic and specified; different orders must not silently change proof identity.

## 5. Issuance and admission contract

~~~text
untrusted source / artifact / proof claims
    -> parse and type-check candidate (untrusted)
    -> trusted loader supplies immutable source bytes or validated manifest
    -> verifier validates source-set identity and recognized specification version
    -> verifier checks each proposition, witness origin, rule and parent dependency
    -> reject cycles, wrong operation, stale epoch and cross-context substitution
    -> mint typed session-local handle in trusted registry
    -> lowerer/VM consumes only this handle for the exact authorized operation
    -> invalidate on session/epoch/policy/source-set boundary
~~~

A verifier cannot prove the correctness of a type derivation from an unvalidated claim or an arbitrary SHA-256 label alone. It must either independently check an admissible derivation against trusted inputs or rely on a separately defined and qualified trusted attestation boundary. This choice requires an upstream normative decision. The verifier must not simply trust a compiler-emitted "valid=true" bit.

Handle safety:
- Handles are typed, issuer-scoped and immutable; there is no public constructor, with update, field extractor, cast, trusted serialize/deserialize or repr reinterpretation.
- Issued handle IDs need not be secret; an untrusted program may not materialize or pass an arbitrary raw ID as a VerifiedEvidence value.
- On each use, the trusted registry checks type, issuer/session, source set, epoch, policy namespace, authorized operation and scope.
- Unsupported verifier versions and proof rules fail closed. No silent fallback to a public Evidence record.
- Compiler-internal Evidence remains an **untrusted data carrier** until independent admission.

## 6. Candidate source-set manifest contract (not frozen)

Require one canonical serialization before choosing digest and proof transport. Proposed order:

1. Manifest version and domain tag, compiler contract identity, source-root logical identity.
2. A sorted set of unique *logical* source paths relative to the declared root. Reject empty/absolute/traversal paths, duplicate logical IDs and aliases that resolve to the same file in the supported platform profile.
3. For each file: length-prefixed logical path, raw byte length and raw byte digest; no platform newline, BOM or Unicode normalization of the **contents**.
4. Import graph with deterministic parent/child IDs and ordered import-edge positions, using source-local byte offsets.
5. Domain-separated SHA-256 over the complete length-prefixed canonical serialization.

Portable path specification remains an explicit design gate. Decide slash conversion, case-folding collision policy, symlinks and filesystem canonicalization, invalid UTF-8 path representations, Unicode equivalence and platform-specific roots before declaring cross-platform identity. Runtime checking should consume the captured immutable bytes, not reopen physical paths after the issuance decision. Hash algorithm versioning and collision handling are part of the protocol; signatures and author identity are out of scope.

## 7. Language, verifier and compiler ownership boundaries

**Upstream Semantic (separate owner authorization and PR):**
- Nominal opaque value type and construction/inspection restrictions.
- Verifier-recognized proof claim / handle admission mechanism.
- VM runtime value isolation and invalid-handle traps.
- Versioned, opt-in ABI contract with exact unsupported-version behavior.
- Source- and raw-SemCode-level negative tests.

**Semantic-Language (after qualified upstream contract exists):**
- Build manifest from canonical source snapshots/import graph.
- Bind typed obligations, file-qualified witnesses and lowering authorization.
- Transfer rules through IR transformations, preserving source lineage.
- Consumer integration in driver/lowering; unchanged legacy production output on old C0.
- Differential and mutation tests using an exact qualified upstream SHA.

Do not introduce a private Semantic host callback that performs hidden typechecking, parsing, lowering or SemCode generation. The host remains the authority for verified execution and externally specified pure mechanics, not secret compiler-domain implementation.

## 8. Red-team acceptance matrix

| Case | Expected |
| --- | --- |
| Manual VerifiedEvidence literal; with mutation | source rejection; raw SemCode rejection |
| Field extraction, pattern extraction, cast, fabricated handle number | fail closed |
| Fabricated T or S without matching witnessed derivation | reject |
| T+F for same proposition | S, never T authorization |
| JOIN across proposition kind/owner/scope/obligation | reject |
| Duplicated, reordered against canonical policy or substituted witnesses | reject |
| Cross-file same-offset substitution | reject |
| Same paths, modified bytes | manifest mismatch/reject |
| Same bytes, other source-set/import graph | context mismatch/reject |
| Changed AST after proof issuance | reject |
| Wrong operation, epoch, policy, compiler/spec version | reject |
| Unknown transfer rule, cyclic/invalid parents | reject |
| Fabricated SemCode proof section, old header with new claim | reject |
| Repeat build with same immutable inputs | stable canonical identity and result |
| OS path spelling and mixed CRLF/LF fixtures | declared portable policy only; deterministic or explicit reject |
| Existing Q3.9/Q4.9 golden case | byte-for-byte unchanged under old frozen C0 |

All expected failures should be typed/classified. Avoid the blanket "error unsupported" if it obscures whether rejection happened at parse, admission, verifier or runtime. No positive authorization test may use a synthetic self-signed claim as its sole oracle.

## 9. Implementation gates and change-control

**Q3.10-A (this PR):** candidate contract, threat model, decision matrix and test plan only. No real proof issuance and no security-completion claim.

**Q3.10-B:** upstream owner-approved language/verifier/VM contract and independent proof-of-rejection tests. Frozen C0 and existing SemCode headers remain unchanged until explicitly governed migration.

**Q3.10-C:** separate, narrow compiler integration PR using an exact qualified upstream reference. Preserve Q3.9 canonical Sema replay and Q4.9 provenance during transition. No automatic promotion.

**Q3.10-D:** negative/raw-SemCode/mutation/differential/cross-platform qualification; identify any expected baseline divergence such as n24 explicitly.

**Q3.10-E:** self-hosting C1/C2 and Bootstrap Seal only under their own authority; cannot be inferred from Q3.10-A/B/C.

The active Harness permits docs/**, compiler/** and tests/** on the bootstrap repo but sets language_implementation=false and forbids .harness/**, .github/**, scripts/harness-check.ps1, bootstrap/** and reference/**. An architectural need cannot self-expand this scope. No merge without owner GO.

## 10. Owner decisions still required

1. **Approve issuer model:** verifier-minted opaque typed handle (recommended) versus stronger alternative with independently verifiable attestations.
2. **Approve portable source identity:** exact canonical manifest encoding/path policy and SHA-256 digest semantics.
3. **Approve proof transport:** separately versioned SemCode section versus separately versioned verifier-admission envelope. Internal sidecars alone cannot satisfy the hostile-SemCode goal.
4. **Approve trust anchor:** verifier recomputation/validation of derivation, or explicitly qualified issuer attestation and its threat model.
5. **Approve rollout:** upstream compatibility plan, pinned C0 transition, tests and fixed-point timing.

Until these decisions and downstream verification are complete, Q3.10 is a design candidate only. No new feature is described as implemented.
