# Self-hosting workflow

Status: **Bootstrap Contract v4 workflow. `C1 == C2` (`byte-equality-v1`) holds. Bootstrap Seal NOT accepted (independent reproduction and owner acceptance pending).**

The Semantic compiler in [`compiler/`](../../compiler/) compiles itself. The pinned C0 compiles the
nine-module source set `S` into C1. C1, executed from its stored and verified bytes, compiles `S`
into C2, and C2 compiles `S` into C3. In this tree **C1, C2 and C3 are byte-identical**, so the
contract comparison `C1.smc == C2.smc` (`byte-equality-v1`) holds. This rests on the C0
code-generation parity described in [C0_PARITY.md](C0_PARITY.md).

Under [Bootstrap Contract v4](../BOOTSTRAP_CONTRACT.md) (§10) a passing attempt produces a contract
evidence record (`report.json` → `bootstrap_record`) that the canonical gate `check_evidence` validates.
That is a *validated* Seal candidate, not an accepted Seal: acceptance needs an independent reproduction
(distinct host and operator) and the owner's acceptance (§10.9). See [Bootstrap Seal](#bootstrap-seal).

## Quickstart

```text
python -I tests/selfhost/selfhost.py all --out <new directory outside the repository> \
    --smc <smc.exe built from C0 1d1e76c7> --executor <smc.exe built from the [executor] revision> \
    --c0-checkout <clean skulmakov-oss/Semantic checkout at 1d1e76c7> [--reproduction-build]
```

- **Run time:** about 20 minutes on the reference host. The two self-compilations take about 5.5 minutes each.
- **Result:** the exit status is 0 only if every phase and the result checker pass.
- **Where things go:** a summary is written to `<out>/summary.md` and the verdict to `<out>/report.json`.

Every input can also come from an environment variable: `SEMANTIC_C0_SMC`, `SEMANTIC_EXECUTOR`,
`SEMANTIC_C0_CHECKOUT`, `SEMANTIC_S2_HARNESS` and `SEMANTIC_REV5_DRIVER`. No path is built in. Each
tool is identified by the SHA-256 pinned in [`tests/selfhost/pins.json`](../../tests/selfhost/pins.json);
the run refuses any other binary. The executor is also identified by its source revision: `smc version
--json` must report the pinned `source_fingerprint`. Builds embed their path, so a rebuild of the
same revision on another host has a different SHA-256. `--reproduction-build` accepts such a binary when
its self-reported source fingerprint matches (a consistency check, not proof); that attempt yields
reproduction identities for a Seal candidate (`facts.seal_reproduction`), never a v4 qualification record.

## Generation chain

```text
S  = compiler/{ast,bytes,driver,lexer,lower,main,parser,sema,semcode}.sm
     identity: shf0-source-set-v1 framing over the files listed in pins.json

C0 smc compile S                                          -> C1   (Rust code generator, pinned C0)
executor smc run-smc stored C1 --expect-sha256 (cli-read-only) S  -> C2   (Semantic compiler)
executor smc run-smc stored C2 --expect-sha256 (cli-read-only) S  -> C3   (Semantic compiler)
sm-verify (C0 and executor) admits C1, C2 and C3; C1 == C2 == C3 is compared byte for byte
```

The host transports bytes only. It reads the declared files and runs `smc`, the executor and
optionally `s2-harness`. It decodes the compiler's `hex …` line and compares bytes. All
compiler-domain work runs as Semantic code: lexing, parsing, analysis, lowering and SemCode emission.

The executor runs every self-compilation with `--envelope trusted-compiler --max-steps 150000000
--max-calls 2000000 --max-frames 256` (the v4 envelope); the checker requires exactly these effective
quotas in the executor's own metrics.

**Optional engineering containment (`--s2`, not part of the v4 protocol).** With `--s2` (an S2 build
pinned in `pins.json`) and `--driver` (the rev5 driver), every self-compilation runs under the S2
harness (K1 job, K2 child-process policy, K3 Low integrity) instead; S2 wraps the rev5 driver, not the
v4 executor. Such an attempt is containment evidence, not a v4 qualification, and S2 itself is not
normatively qualified.

## Phases

| Phase | Produces | Fails closed when |
|---|---|---|
| `preflight` | `preflight.json`, `snapshot/` | a tool hash differs from `pins.json`; the C0 checkout is not the pinned SHA or is dirty; any source file violates `shf0-source-set-v1` (paths, UTF-8, LF only, final newline) |
| `build` | `build.json`, `gen/C1..C3.smc` | C0 compilation fails; a generation is not admitted by C0's or the executor's `sm-verify`; the executor did not run the declared previous generation to completion under the declared envelope; the snapshot changes |
| `verify` | `verify.json` | `C1 != C2` or `C2 != C3` (`--fixed-point c2c3` checks only post-bootstrap stability, for the historical reproduction) |
| `differential` | `differential.json`, `fixtures/`, `mutations/` | any fixture misses its expectation (see below) |
| `report` | `report.json`, `summary.md` | the independent checker finds any violation |

Every command is logged in `commands.jsonl`, with byte-exact stdout and stderr under `logs/`.
Files are created exclusively: nothing is overwritten or deleted, and there are no automatic retries.

## Differential corpus

- **Corpus:** all 134 cases of [`tests/compiler/cases.json`](../../tests/compiler/cases.json) and the
  six import fixtures in `tests/compiler/imports/`.
- **Isolation:** each fixture is copied into its own source root, so imports cannot reach outside it.
- **Runs:** each fixture is compiled by C0 (`smc compile`) and by C1 and C2 executed through the executor,
  and once more by C2 to check determinism.

| Expectation | Passes only if |
|---|---|
| `ok` | C1 and C2 emit SemCode **byte-identical to C0's**; `sm-verify` admits it; it runs cleanly on C0's `smc run-smc` and on the executor (profile `pure`) |
| `trap:<text>` | as `ok`, but both runs must trap with `<text>` |
| `reject:<category>:<text>` | the compiler run completes and prints that diagnostic; no artifact |
| `fault:<text>` | the compiler process faults with `<text>` (the missing-import `ABI FsRead failure`); no artifact, no diagnostic |

Every fixture also requires identical output from C1, C2 and the repeat.

- **Source mutation proof:** two compiler-source mutations are compiled.
  - A comment-only edit changes the source-set identity but not C1.
  - A changed diagnostic text changes both.

## Verifying an evidence package

```text
python -I tests/selfhost/check_results.py <attempt>                          # recompute every verdict
python -I tests/selfhost/check_results.py --selftest <attempt> <new scratch>  # fault injection
python -B -m unittest discover -s tests/selfhost -p "test_*.py"                # binary-free unit tests
```

The checker trusts no recorded verdict and is independent of `selfhost.py` (it has its own framing,
classification, import expectations and mutation table). It recomputes from the raw files:
- **Inventory and logs:** the exact file inventory and the log hashes.
- **Commands and tools:** the exact set of commands, each compared with the full command line rebuilt
  from `pins.json`, the recorded attempt root and the corpus (tool, subcommand, input, output, root,
  envelope and entry). Each pinned tool, including the pinned S2 build, is re-hashed when it is still present.
- **C0 attestation:** the C0 checkout is attested from the logged `git rev-parse HEAD` and
  `git status` output. That is an attestation, not cryptographic proof.
- **Snapshot:** the source-set identity of the snapshot.
- **Generations:** each generation is decoded again from the producing run's stdout and bound to the
  executor provenance of the previous generation. Under S2, the accepted record must publish exactly
  that artifact.
- **Comparisons:** the byte comparisons.
- **Corpus:** it must equal this repository's `tests/compiler/cases.json` and import fixtures, byte for
  byte. Verify an old package from the commit that produced it.
- **Fixture verdicts:** every fixture run is bound to its generation, and every verdict is recomputed
  from stdout, the executor's metrics and both VMs' logs.
- **Mutations:** the two pinned source mutations are re-applied to the snapshot.

The self-test injects 30 known violations, plus 3 more for S2 attempts:
- forged artifact, log, tool and C0 records;
- a verifier rejection, and a verify command that ran something else;
- an exit-0 diagnostic, and a fixture run executed by the wrong generation;
- removed, duplicated and dropped rows or cases, and rewritten case, import and mutation expectations;
- an altered mutation source and an altered trap log;
- a mutated snapshot, a forged verdict and a failed VM run;
- a deleted, `null` or single-generation differential phase, and an unexpected extra command;
- an executor run that did not complete, ran under another envelope, a forged executor fingerprint and a missing executor admission;
- a clean attempt whose contract record the canonical `check_evidence` rejects (the gate's verdict is part of the checker's);
- for S2, a non-accepted record, a stray file inside the S2 attempt and a verification aimed at another attempt.

It requires every one to be rejected and the unmodified copy to pass.

## Successful engineering self-host

An attempt is an engineering success when `report.json` has `"engineering_self_host_success": true`.
That requires `PASS`, the full scope (all phases) and the default `--fixed-point c1c2`. A `PASS` with
`--fixed-point c2c3` or `--partial` (the historical reproduction) only says the evidence matches that
narrower claim. Engineering success means:
- C1, C2 and C3 are admitted, and C1 == C2 == C3;
- every corpus expectation holds, with C0-identical SemCode for every accepted program;
- compiler output is deterministic;
- both source mutations behave as stated.

## Bootstrap Seal

[Bootstrap Contract v4](../BOOTSTRAP_CONTRACT.md) binds the canonical source set
(`bootstrap/source-set.toml`, the same nine files as `pins.json`), the stored-artifact executor
(`[executor]`: `smc run-smc` at an exact `skulmakov-oss/Semantic` revision), the trusted-compiler
envelope and the evidence record. `report.json` carries the record built from checker-verified facts
and the result of `check_evidence` on it (a rejected record fails the attempt); `python -I bootstrap/validate_contract.py --check-evidence
<record.json>` repeats that check.

A Seal candidate (`shf0-bootstrap-seal-v1`) moves through three states, checked with
`--check-seal <seal.json>` (§10.9):
1. **validated:** the record passes `check_evidence`;
2. **reproduced:** an independent reproduction on a distinct host by a distinct operator, from the
   same C0 and `S`, reports the same identities (run this workflow there with `--reproduction-build`);
3. **accepted:** the owner accepts it.

A second run on the same host or by the same operator is repeatability evidence, not a reproduction.
Cross-platform qualification is SHF-17 and is not part of this Seal.

## Known limitations and failure taxonomy

- **Parity scope.** C0 parity is shown on the corpus and on `S` ([C0_PARITY.md](C0_PARITY.md#limits)).
- **Missing import.** A missing import is a compiler **process fault**: `ABI FsRead failure` from the host read capability, not a compiler diagnostic. This is unchanged and classified separately.
- **Fixtures outside S2.** Fixture compilations use the executor directly; with `--s2` only the generation chain runs under S2.
- **Failure classes.** Failures are reported with the contract's classes where they apply:
  - `REFERENCE_MISMATCH`: a tool or C0 identity differs;
  - `SOURCE_SET_INVALID`: a preflight source error;
  - `ADMISSION_REJECT`: `sm-verify` rejects an artifact;
  - `FIXED_POINT_DELTA`: `verify` fails;
  - `NONDETERMINISM`: a repeat differs.
