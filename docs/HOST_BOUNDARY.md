# Host Boundary

Status: **policy boundary frozen at architecture level — no operation set frozen**

Salvaged from PR #6 (`docs/HOST_CAPABILITY_ABI.md` @ `3714ce27`). Renamed to `HOST_BOUNDARY`
because no executable ABI (opcodes, wire tags, API spellings) is frozen here. This document
freezes the **policy**, not invented operations. Concrete host capabilities remain owned by the
PROMETHEUS boundary in `skulmakov-oss/Semantic`.

## 1. Core rule

> The host performs mechanics. Semantic performs compiler meaning.

Semantic code obtains no ambient host authority. Every externally observable host effect crosses
an explicit, capability-gated boundary.

```text
Semantic compiler logic (S)
          |
          v
declared host capability
          |
          v
platform adapter (mechanics only)
          |
          v
filesystem / process / OS
```

## 2. The host may

- read declared input bytes;
- write declared artifact bytes;
- expose declared capabilities;
- transport arguments and results;
- execute the existing verifier and VM.

## 3. The host may not

- tokenize;
- parse;
- bind symbols;
- typecheck;
- lower;
- choose compiler IR;
- select compiler opcodes;
- construct Semantic compiler output;
- reinterpret diagnostics;
- bypass admission.

Any evidence produced through a path that does one of these is invalid (`HOST_LOGIC_LEAK` in
[BOOTSTRAP_CONTRACT.md](./BOOTSTRAP_CONTRACT.md)).

## 4. Capability principles

Every admitted host operation is: explicitly named; versioned; bounded; deterministic in its
observable request/result; explicit about failure; never fail-open. **Absence of a capability
means denial, not fallback.** No hidden retry may switch to a more privileged operation.

## 5. Effect classes needed by the first bootstrap

Categories only; none is frozen.

- **Source input** — obtain declared source bytes. The host must not reorder enumeration; any
  canonical ordering is applied by Semantic-owned logic.
- **Artifact output** — write produced bytes exactly. Artifacts must not acquire timestamps,
  absolute paths, random IDs or other undeclared metadata.
- **Declared environment inputs** — default **none**. A needed value becomes a declared input.

## 6. Determinism

For identical declared inputs and identical capability results, compiler output is identical
under the frozen comparison. Host adapters must not inject: wall-clock time, absolute host paths,
directory enumeration order, randomized ordering, locale-sensitive transformation, undeclared
environment state, process/thread identity, or scheduling-dependent output.

## 7. Error model

```text
request -> capability check
             -> denied   => explicit denial
             -> admitted -> host op -> success => explicit result
                                    -> failure => explicit error
```

A failure is never translated into a plausible successful value.

## 8. Freeze criteria for a concrete capability

A capability becomes frozen only when: upstream owner and SHA are identified; request/result
representation, denial and failure behavior are specified; determinism and bounds are specified;
positive, denied, malformed and boundary tests exist; and the adapter contains no compiler-domain
logic. If the capability changes the runtime contract, it is defined upstream first.
