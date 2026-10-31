# Q4.9 — File-Qualified Source Provenance and Verified IR Lineage (experimental)

**Date:** 2026-10-31. **Authority:** local worktree only. Frozen C0/VM/Verifier/Bootstrap Contract and SemCode encoding remain outside scope.

## Invariant and identity model

A semantic witness's `Node.pos` is a UTF-8 byte offset in the **expanded compilation unit**, not necessarily its original file. Q4.9 establishes a deterministic *source snapshot* identity consisting of a loader-controlled logical path **and the exact complete UTF-8 source text**. This is an exact textual identity, **not a cryptographic content hash, authenticated path or signature**.

`main.expand` now returns both the unchanged flattened text and two sidecar collections:

- `SourceFile(path, content)`: the full text as read for every participating source file, including nested direct-local imports.
- `SourceSpan(path, local_start, flat_start, size)`: one continuous segment copied from that file into the expanded stream. Loader-generated delimiters use `path = ""` and have no file-qualified origin.

The loader preserves original import restrictions and reconstruction order. A child span's `flat_start` is shifted by the exact current expanded length. Byte-offset mapping is valid across nested imports, comments, CRLF/LF and EOF boundaries.

## Source admission and AST authenticity (local trust boundary)

`compile_source_with_origins(src, files, spans)` is the file-backed entrypoint used by the loader. `compile_source(src)` remains a compatible in-memory entrypoint using a synthetic `<memory>` identity.

Before Sema, `q4_source_bundle_error` verifies that every span is contiguous, nonempty, within bounds, backed by a unique known file, and byte-for-byte equal to the corresponding source snapshot substring. A generated span may contain only LF or CRLF. Source-file contents are **re-read** and compared to snapshots for physical files, rejecting changes between loading and admission. This does not eliminate an adversarial race after the check.

After ordinary parsing, `q4_parsed_identity_error` lexes/parses the exact expanded source snapshot **again** and checks all AST node fields, ordered children, function signatures/bodies, and record descriptors. The Q3.9 trusted Sema replay then performs its own independent canonical type-evidence reconstruction.

## File → AST → Sema → IR trace

`q4_trace_function` operates on the **Q3.9-admitted canonical Sem** and its freshly lowered non-emitted `FnIr`. For every emitted-in-memory `PhaseProof`, it checks exact positive-witness order, owner/proposition identity, AST node ID, Node.pos and equality with a canonical Sema carrier. The same validation covers `BindingInferenceLink`, `TupleInferenceLink` and `ControlTypeCheck` with their corresponding metadata.

Each checked link produces a `ProvenanceStep(stage, proposition, owner, source_node, path, file_offset, flat_offset)`. `q4_origin_at` resolves the flat byte position to a file and byte offset; it **rejects** references to loader-generated separators or unmapped positions. The driver refuses SemCode emission if a lower-level link check fails.

Supported stages are `evidence-to-ir`, `type-inference-to-ir`, `control-type-to-ir`. They describe validated boundaries; the traces are non-emitted sidecar values, not stable machine-checked proofs inside the encoded program.

## Qualification targets

- New internal fixture `tests/compiler/programs/quad_source_provenance_internal.sm`: two distinct file snapshots and AST→IR proof traces; exact offset correspondence; source-file contents replacement, duplicate file identities, noncontiguous/gapped spans, incorrect local source byte offset, missing source span, synthetic separator rejection, tampered IR witness and tampered AST node position.
- New `test_quad_native.py` tests execute the real fixture and three actual loader scenarios: direct import, nested mixed-CRLF import, and helper ending in a comment without LF. Positive artifacts must be stable, verifiable and executable by pinned C0.
- Pre-existing `test_imports.py` covers direct-local import correctness, duplicate/cyclic import errors, source root protection, comments and large inputs.

## Trust and performance limits

Q4.9 does **not** cryptographically authenticate source files or verify filesystem ownership. Re-reading source bytes is TOCTOU-sensitive; the source registry, parser and loader are within the current trusted implementation. A caller can still construct raw `Evidence`, `Sem`, `SourceFile` and `ProvenanceStep`; production `compile_source_with_origins` validates the construction path, not arbitrary external records.

IR origin chains stop at internal `FnIr`; there is **no stable optimizer-transformation ancestry per instruction**, no SemCode proof section, no frozen verifier-facing proof ABI, no canonical multi-file source-set digest, and no durable cryptographic seal. A `path + exact bytes` snapshot is collision-free as text equality but is not a compact/cross-system authenticated file identifier. Platform-stable path canonicalization remains a governed future task.

Performance is intentionally conservative: a second lexer/parser pass, a second Sema pass (Q3.9), source-byte verification and proof-to-source tracing. Optimization requires separately controlled measurements and must retain fail-closed semantics.

See `EVIDENCE_ADT_DESIGN_Q3_10.md` for a separately proposed truly closed-carrier model; it is not implemented by Q4.9.
