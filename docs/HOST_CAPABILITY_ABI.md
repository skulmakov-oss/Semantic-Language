# Semantic Host Capability ABI

Status: **bootstrap boundary specification — operation set not yet frozen**

This document defines the policy boundary between Semantic-owned compiler/runtime logic and host/platform effects during bootstrap.

It is intentionally narrower than a general operating-system syscall interface.

## 1. Core rule

Semantic semantics do not obtain ambient host authority.

All externally observable host effects must cross an explicit, versioned, capability-gated boundary.

Conceptually:

```text
Semantic compiler / runtime logic
              |
              v
     Host Capability Contract
              |
              v
       platform adapter
              |
              v
      filesystem / process /
      environment / other OS
```

The platform adapter provides mechanics, not language semantics.

## 2. Non-authority rule

The host boundary must never perform hidden compiler-domain work.

Forbidden host-side fallbacks include:

- tokenization;
- parsing;
- name resolution;
- type checking;
- lowering;
- opcode selection;
- SemCode layout decisions;
- verifier bypass/admission override.

A host adapter may move declared bytes or expose declared capabilities. It may not decide what Semantic source means.

## 3. Capability principles

Every admitted host operation must satisfy all of the following:

- explicitly named capability;
- versioned contract;
- deterministic request representation where observable;
- bounded inputs/outputs where applicable;
- explicit error result;
- no fail-open behavior;
- no hidden environment dependence;
- auditable invocation where the surrounding runtime contract requires audit;
- revocable/denyable authority where the capability model permits it.

Absence of a capability means denial, not fallback.

## 4. Bootstrap effect classes

The first self-hosting compiler may require a minimal set of host-effect classes.

This document does **not** yet assign final opcode numbers, wire tags, or public API spellings.

### 4.1 Source input

Purpose: obtain declared compiler/project input bytes.

Required contract questions before freeze:

- how a source object is identified;
- whether identity is path-based, content-based, manifest-based, or another explicit form;
- exact byte representation;
- maximum/bounded read behavior;
- deterministic error representation;
- normalization rules, if any.

The host must not reorder source enumeration implicitly.

### 4.2 Artifact output

Purpose: write explicitly produced compiler artifacts.

Required contract questions before freeze:

- artifact identity/name;
- overwrite/atomicity policy;
- exact bytes written;
- error representation;
- whether paths are semantic inputs or host-only destinations.

Artifact contents must not acquire timestamps, absolute paths, random IDs, or other undeclared host metadata.

### 4.3 Declared environment inputs

Default policy: **none**.

If a bootstrap compiler legitimately needs an external environment value, that value must become an explicit declared input/capability and therefore part of reproducibility/invalidation evidence.

Ambient reads of time, locale, random state, process ID, current directory, or arbitrary environment variables are forbidden in the deterministic compiler path unless an explicit future contract admits them.

### 4.4 Memory/resources

Ordinary language/runtime memory management is not automatically a host capability.

If allocation is internal to the admitted runtime, its implementation remains runtime mechanics and must preserve deterministic observable semantics.

If an external allocator/resource service becomes part of the boundary, it requires its own explicit capability contract and quota behavior before use.

## 5. Determinism and reproducibility

For identical declared Semantic inputs and identical admitted capability results, compiler-observable output must be equivalent under the applicable canonical comparison.

Host adapters must not inject:

- wall-clock timestamps;
- host-specific absolute paths;
- nondeterministic directory enumeration;
- randomized ordering;
- locale-sensitive transformation;
- undeclared environment state;
- process/thread identity;
- scheduling-dependent output.

If platform mechanics naturally return unordered data, the Semantic-owned layer must apply the frozen canonicalization rule before the data can influence canonical output.

## 6. Error model

Every host operation must have an explicit failure representation.

The boundary must not translate failure into a plausible successful value.

Conceptually:

```text
request
  -> capability check
      -> denied        => explicit denial
      -> admitted
           -> host op
               -> success => explicit result
               -> failure => explicit error
```

No hidden retry/fallback may switch to a more privileged operation.

## 7. Capability identity and admission

Capability identity/version is part of the execution context wherever it can affect verifier admission, runtime behavior, or compiler reproducibility.

A cached admission/proof must not be reused if the relevant capability contract has changed.

The stricter incremental-admission closure is governed by the Self-Hosting + Instant Pipeline milestone.

## 8. Initial capability registry

No concrete capability spelling is frozen yet.

SH-00 / the authority-manifest work must derive the smallest required bootstrap set from the reference repository and compiler source requirements.

| ID | Capability/effect | State | Authority | Determinism notes |
|---|---|---|---|---|
| HC-000 | host capability ABI baseline | CANDIDATE | to be pinned by SH-00 authority manifest | policy placeholder; not an executable capability |

Potential effect classes such as source input and artifact output remain **unfrozen categories** until their exact reference contracts are identified.

## 9. B0/B7 relationship

B0 Foundation itself remains source-independent and host-effect free.

This ABI exists so later compiler/tooling/runtime work does not solve required effects by smuggling platform behavior into Foundation.

B7 VM/runtime qualification must prove that observable host interaction crosses an explicit boundary rather than hidden platform calls.

## 10. Freeze criteria

A host capability may become FROZEN only when:

- owner/reference authority is identified;
- request/result representation is specified;
- denial and failure behavior are specified;
- determinism/canonicalization rules are specified;
- quota/bounds are specified where relevant;
- audit behavior is specified where relevant;
- positive, denied, malformed, and boundary tests exist;
- no compiler-domain semantic authority is implemented by the host adapter.

The bootstrap should prefer the smallest host capability surface that can support the proven compiler path.
