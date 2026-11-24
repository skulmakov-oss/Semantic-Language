# Self-hosting workflow

Status: **ENGINEERING. Contract-v3 byte equality `C1 == C2` measured. Bootstrap Seal NOT established.**

The Semantic compiler in [`compiler/`](../../compiler/) compiles itself. The pinned C0 compiles the
nine-module source set `S` into C1. C1, executed from its stored and verified bytes, compiles `S`
into C2, and C2 compiles `S` into C3. In this tree **C1, C2 and C3 are byte-identical**, so the
protocol-v3 comparison `C1.smc == C2.smc` (`byte-equality-v1`) holds. This rests on the C0
code-generation parity described in [C0_PARITY.md](C0_PARITY.md).

What this is **not**: a Bootstrap Seal, a normative SHF-15/SHF-16 qualification, an independent
reproduction or a cross-platform result. See [What the Seal still needs](#what-the-bootstrap-seal-still-needs).

## Quickstart

```text
python -I tests/selfhost/selfhost.py all --out <new directory outside the repository> \
    --smc <smc.exe built from C0 1d1e76c7> --driver <rev5 stored-exec-driver.exe> \
    --c0-checkout <clean skulmakov-oss/Semantic checkout at 1d1e76c7> [--s2 <s2-harness.exe>]
```

- **Run time:** about 25 minutes on the reference host. The two self-compilations take about 7 minutes each.
- **Result:** the exit status is 0 only if every phase and the result checker pass.
- **Where things go:** a summary is written to `<out>/summary.md` and the verdict to `<out>/report.json`.

Every input can also come from an environment variable: `SEMANTIC_C0_SMC`, `SEMANTIC_REV5_DRIVER`,
`SEMANTIC_C0_CHECKOUT` and `SEMANTIC_S2_HARNESS`. No path is built in. Each tool is identified by the
SHA-256 pinned in [`tests/selfhost/pins.json`](../../tests/selfhost/pins.json); the run refuses any
other binary.

## Generation chain

```text
S  = compiler/{ast,bytes,driver,lexer,lower,main,parser,sema,semcode}.sm
     identity: shf0-source-set-v1 framing over the files listed in pins.json

C0 smc compile S                                  -> C1   (Rust code generator, pinned C0)
rev5 driver executes stored C1 (cli-read-only) S  -> C2   (Semantic compiler)
rev5 driver executes stored C2 (cli-read-only) S  -> C3   (Semantic compiler)
sm-verify admits C1, C2 and C3; C1 == C2 == C3 is compared byte for byte
```

The host transports bytes only. It reads the declared files and runs `smc`, the driver and
optionally `s2-harness`. It decodes the compiler's `hex …` line and compares bytes. All
compiler-domain work runs as Semantic code: lexing, parsing, analysis, lowering and SemCode emission.

With `--s2` (an S2 build whose SHA-256 is pinned in `pins.json`), every self-compilation runs under the S2 harness (K1 job, K2 child-process policy,
K3 Low integrity). Acceptance is decided by `s2-harness --verify-attempt`, and the published
output must equal the generation artifact.

## Phases

| Phase | Produces | Fails closed when |
|---|---|---|
| `preflight` | `preflight.json`, `snapshot/` | a tool hash differs from `pins.json`; the C0 checkout is not the pinned SHA or is dirty; any source file violates `shf0-source-set-v1` (paths, UTF-8, LF only, final newline) |
| `build` | `build.json`, `gen/C1..C3.smc` | C0 compilation fails; a generation is not admitted by `sm-verify`; the executor's provenance does not name the previous generation's hash; the snapshot changes |
| `verify` | `verify.json` | `C1 != C2` or `C2 != C3` (`--fixed-point c2c3` checks only post-bootstrap stability, for the historical reproduction) |
| `differential` | `differential.json`, `fixtures/`, `mutations/` | any fixture misses its expectation (see below) |
| `report` | `report.json`, `summary.md` | the independent checker finds any violation |

Every command is logged in `commands.jsonl`, with byte-exact stdout and stderr under `logs/`.
Files are created exclusively: nothing is overwritten or deleted, and there are no automatic retries.

## Differential corpus

- **Corpus:** all 134 cases of [`tests/compiler/cases.json`](../../tests/compiler/cases.json) and the
  six import fixtures in `tests/compiler/imports/`.
- **Isolation:** each fixture is copied into its own source root, so imports cannot reach outside it.
- **Runs:** each fixture is compiled by C0 (`smc compile`) and by C1 and C2 executed through the driver,
  and once more by C2 to check determinism.

| Expectation | Passes only if |
|---|---|
| `ok` | C1 and C2 emit SemCode **byte-identical to C0's**; `sm-verify` admits it; it runs cleanly on `smc run-smc` and on the driver (profile `pure`) |
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
  from stdout, the driver provenance and both VMs' logs.
- **Mutations:** the two pinned source mutations are re-applied to the snapshot.

The self-test injects 25 known violations, plus 3 more for S2 attempts:
- forged artifact, log, tool and C0 records;
- a verifier rejection, and a verify command that ran something else;
- an exit-0 diagnostic, and a fixture run executed by the wrong generation;
- removed, duplicated and dropped rows or cases, and rewritten case, import and mutation expectations;
- an altered mutation source and an altered trap log;
- a mutated snapshot, a forged verdict and a failed VM run;
- a deleted, `null` or single-generation differential phase, and an unexpected extra command;
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

## What the Bootstrap Seal still needs

These items are outside the authority of an ordinary engineering PR. [Bootstrap Contract v3](../BOOTSTRAP_CONTRACT.md) stays as it is.

1. **Source-set manifest.**
   - `bootstrap/source-set.toml` still has `files = []`, and `bootstrap/**` is protected.
   - Setting it to the nine files in `pins.json` takes an owner-authorized governance transition.
   - Until then, `check_evidence` cannot bind a record to the real `S`.
2. **Execution envelope.**
   - §10.5 declares the trusted-compiler envelope as 20,000,000 steps and 50,000 calls.
   - Producing C2 needs about 100.8M steps and 1.09M calls; the workflow uses 150M / 2M / 256.
   - Running `C1(S)` inside the declared envelope is impossible, so the contract's numbers need an owner decision.
3. **Stored-artifact executor.**
   - C2 is produced by the rev5 `stored-exec-driver`, which pins C0's `sm-vm` and `sm-verify` crates.
   - C0's own CLI cannot execute a stored `.smc` under the trusted-compiler envelope, and rev5 is not upstream-admitted.
4. **Independent reproduction and owner acceptance** (§10.7–§10.8).
   - All evidence so far comes from one Windows host, one operator and AI review.
5. **Cross-platform qualification** (SHF-17).

## Known limitations and failure taxonomy

- **Parity scope.** C0 parity is shown on the corpus and on `S` ([C0_PARITY.md](C0_PARITY.md#limits)).
- **Missing import.** A missing import is a compiler **process fault**: `ABI FsRead failure` from the host read capability, not a compiler diagnostic. This is unchanged and classified separately.
- **Driver-only fixtures.** Fixture compilations use the driver directly; only the generation chain runs under S2.
- **Failure classes.** Failures are reported with the contract's classes where they apply:
  - `REFERENCE_MISMATCH`: a tool or C0 identity differs;
  - `SOURCE_SET_INVALID`: a preflight source error;
  - `ADMISSION_REJECT`: `sm-verify` rejects an artifact;
  - `FIXED_POINT_DELTA`: `verify` fails;
  - `NONDETERMINISM`: a repeat differs.
