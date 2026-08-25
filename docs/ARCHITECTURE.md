# Semantic Bootstrap Architecture

Status: bootstrap baseline

## 1. Purpose

`Semantic-Language` exists to move Semantic from a Rust-hosted implementation toward progressive self-hosting without performing a risky whole-system rewrite.

The canonical reference implementation remains:

- `skulmakov-oss/Semantic`

This repository is initially a **shadow implementation**. It becomes canonical only through explicit, evidenced migration milestones.

## 2. System relationship

```text
                REFERENCE ERA

      Semantic source / contracts
                  │
                  ▼
        Semantic (Rust-hosted)
                  │
                  │ oracle
                  ▼
        Semantic-Language bootstrap
                  │
                  ▼
         differential qualification
                  │
                  ▼
            migrated modules

                SELF-HOSTED ERA

         Semantic source
              │
              ▼
     Semantic compiler in Semantic
              │
              ▼
           SemCode
              │
              ▼
      verifier / runtime boundary
```

## 3. Architectural ownership

### Reference repository owns, until migrated

- current language semantics;
- current source grammar and type rules;
- current IR and SemCode behavior;
- verifier admission behavior;
- VM/runtime observable behavior;
- authoritative compatibility decisions.

### Bootstrap repository owns

- self-hosting migration structure;
- Semantic implementations of migrated modules;
- differential adapters and comparison formats;
- bootstrap qualification gates;
- evidence that a migrated module is equivalent to its reference contract;
- explicit records of migration/freeze decisions.

## 4. Non-goals

Bootstrap does **not** require:

- closing every issue in the reference repository;
- replacing Rust all at once;
- reproducing the native Semantic UI stack;
- moving Workbench into the language implementation;
- inventing a second SemCode format;
- changing semantics merely to make bootstrap easier.

UI is outside the bootstrap-critical path. External UI technologies may consume Semantic through stable host/ABI/IPC boundaries independently of language self-hosting.

## 5. Module migration state

Each module has one of five states:

```text
REFERENCE_ONLY
     ↓
MIRRORED
     ↓
DIFFERENTIAL
     ↓
QUALIFIED
     ↓
CANONICAL
```

### REFERENCE_ONLY

Only the Rust-hosted implementation is trusted.

### MIRRORED

A Semantic implementation exists, but equivalence is not yet proven.

### DIFFERENTIAL

Both implementations run against a shared corpus and differences are recorded deterministically.

### QUALIFIED

The bootstrap implementation satisfies the declared qualification matrix for its frozen contract.

### CANONICAL

An explicit migration decision transfers ownership to this repository/module. This transition is never implicit.

## 6. Bootstrap blocking rule

An upstream issue blocks a bootstrap slice only when it may change one of:

1. Semantic meaning;
2. the migrated module's observable public contract;
3. serialized/wire representation consumed or produced by the module;
4. deterministic observable behavior used by qualification.

A blocker is **slice-local** unless evidence shows it invalidates a wider dependency.

## 7. Differential architecture

Preferred comparison shape:

```text
                 canonical input
                      │
             ┌────────┴────────┐
             ▼                 ▼
       Rust reference     Semantic mirror
             │                 │
             ▼                 ▼
       canonical output   canonical output
             │                 │
             └────────┬────────┘
                      ▼
                  comparator
                      │
             equal / explained delta
```

Comparison should use the narrowest stable observable representation available:

- tokens for lexer slices;
- normalized AST for parser slices;
- diagnostics/type facts for semantic slices;
- canonical IR for lowering slices;
- byte-for-byte output where SemCode identity is part of the contract;
- verifier accept/reject diagnostics for admission slices;
- deterministic value/trace results for runtime slices.

## 8. Dependency rule

Bootstrap follows dependency direction. A module must not smuggle a later-layer authority into an earlier slice simply to get tests green.

Target direction:

```text
foundation
   ↓
lexical
   ↓
parser / AST
   ↓
semantic analysis
   ↓
IR / lowering
   ↓
SemCode emission
   ↓
verification
   ↓
runtime / VM
```

The actual sequence may skip ahead where contracts are independent and stable, but dependency inversions require an explicit architecture decision.

## 9. Rust transition model

Rust changes role over time:

```text
Stage 0  canonical implementation
Stage 1  canonical implementation + oracle
Stage 2  oracle + bootstrap host
Stage 3  compatibility/reference Foundation
Stage 4  historical Foundation
```

Rust removal is not itself a success criterion. Proven Semantic ownership is.

## 10. Definition of self-hosting

Semantic may be called self-hosted only after an explicit qualification milestone demonstrates that a sufficiently complete compiler toolchain written substantially in Semantic can process the source needed to reproduce that toolchain under the admitted runtime boundary.

A UI implementation is not part of this definition.
