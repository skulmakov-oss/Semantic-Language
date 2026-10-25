#!/usr/bin/env python3
"""Fail-closed qualification runner for the Semantic-written compiler (compiler/main.sm).

Environment (both required; no default binaries are guessed):
  SEMANTIC_SMC     attested C0 smc-cli that RUNS the Semantic compiler.
  SEMANTIC_C0_SMC  attested pinned C0 compiler/verifier/VM (may be the same binary).
  SEMANTIC_C0_CHECKOUT   clean canonical source checkout at the manifest's exact SHA.
  SEMANTIC_C0_PROVENANCE build/release provenance JSON binding binary and reference.
  SEMANTIC_C0_ANCHOR     upstream anchor JSON binding repository, SHA and lockfile.
Optional:
  SEMANTIC_SMC_PROVENANCE  separate attestation if the host role uses another C0 build;
                          independent Windows builds need not have identical PE hashes.
  SEMANTIC_TEST_OUT   output directory (default: $TMPDIR/semantic-compiler-tests); never inside
                      the source tree.
  COMPILER_MAX_STEPS  Steps budget for running the compiler (default 20000000, trusted-compiler
                      envelope; Calls 50000 and Frames 256).

Per case:
  1. compile with the Semantic compiler twice -> deterministic result line required
  2. "hex ..." -> tests/compiler/hex2bin.py (transport only) -> .smc
  3. verify with BOTH verifiers, run on BOTH VMs (pure profile)
  4. compile with both supplied roles at O0/O1, verify and run (reference behaviour)
  5. compare against cases.json and against the reference outcome
Mutations of generated artifacts must be rejected by both verifiers.
Exit status is nonzero if anything fails.
"""
import hashlib, json, os, pathlib, re, struct, subprocess, sys, tempfile
from reference_identity import qualify_reference
from reference_outcomes import outcome_key, reference_matches, reference_pair_matches

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
COMPILER = REPO / "compiler"


def need_env(name):
    v = os.environ.get(name)
    if not v or not pathlib.Path(v).is_file():
        sys.exit(f"run_tests: {name} must name an existing smc-cli binary")
    return pathlib.Path(v)


SMC = need_env("SEMANTIC_SMC")
C0 = need_env("SEMANTIC_C0_SMC")
try:
    REFERENCE_IDENTITY = qualify_reference(REPO, SMC, C0, os.environ)
except ValueError as error:
    sys.exit(str(error))
OUT = pathlib.Path(os.environ.get("SEMANTIC_TEST_OUT", pathlib.Path(tempfile.gettempdir()) / "semantic-compiler-tests"))
if REPO in OUT.resolve().parents or OUT.resolve() == REPO:
    sys.exit("run_tests: SEMANTIC_TEST_OUT must be outside the source tree")
MAX_STEPS = os.environ.get("COMPILER_MAX_STEPS", "20000000")
if not MAX_STEPS.isdecimal() or not 0 < int(MAX_STEPS) <= 20000000:
    sys.exit("run_tests: compiler Steps must be within the approved 20000000 bound")
for sub in ("native", "reference", "mutations"):
    (OUT / sub).mkdir(parents=True, exist_ok=True)
sha = lambda b: hashlib.sha256(b).hexdigest()


def run(cmd, cwd=None):
    p = subprocess.run([str(c) for c in cmd], cwd=cwd or OUT, capture_output=True,
                       text=True, encoding="utf-8", timeout=1800)
    with (OUT / "commands.jsonl").open("a", encoding="utf-8") as log:
        log.write(json.dumps(dict(command=[str(c) for c in cmd], cwd=str(cwd or OUT),
                                  exit_code=p.returncode, stdout=p.stdout, stderr=p.stderr)) + "\n")
    return p.returncode, p.stdout, p.stderr


def compile_native(case_dir, name):
    rc, so, se = run([SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", case_dir,
                      "--envelope", "trusted-compiler", "--max-steps", MAX_STEPS,
                      "--max-calls", "50000", "--max-frames", "256", "--metrics", "json", "--", name], cwd=COMPILER)
    m = next((json.loads(l) for l in se.splitlines() if l.startswith('{"schema":"semantic.run.metrics')), {})
    return {"rc": rc, "line": so, "outcome": m.get("outcome"), "counters": m.get("counters"), "quota_exceeded": m.get("quota_exceeded")}


def execute(smc, art):
    vrc, vso, vse = run([smc, "verify", art])
    if vrc != 0:
        return {"kind": "rejected_by_verifier", "verify_rc": vrc, "detail": (vso + vse).strip()}
    rrc, rso, rse = run([smc, "run-smc", art])
    msg = (rso + rse).strip()
    return {"kind": "ok" if rrc == 0 else "trap", "verify_rc": vrc, "run_rc": rrc, "detail": msg}


cases = json.loads((HERE / "cases.json").read_text())
reference_cases = json.loads((HERE / "reference_cases.json").read_text())
if set(reference_cases) != {name for name, expected in cases.items() if expected.startswith("reject:")}:
    sys.exit("run_tests: every subset rejection requires an explicit reference expectation")
case_dir = HERE / "cases"
COMPILER_ARTIFACTS = {}
for opt in ("O0", "O1"):
    artifact = OUT / f"compiler.{opt}.smc"
    artifact.unlink(missing_ok=True)
    rc, so, se = run([SMC, "compile", COMPILER / "main.sm", "-o", artifact, "--opt-level", opt])
    if rc or not artifact.exists():
        sys.exit(f"compiler source rejected at {opt}: {so}{se}")
    rc, so, se = run([C0, "verify", artifact])
    if rc:
        sys.exit(f"compiler artifact not admitted at {opt}: {so}{se}")
    COMPILER_ARTIFACTS[opt] = artifact
rows, failures = [], []
for name, expect in sorted(cases.items()):
    src_file = case_dir / f"{name}.sm"
    row = {"case": name, "source_sha256": sha(src_file.read_bytes()), "expect": expect, "checks": {}}
    reference_expect = reference_cases[name] if expect.startswith("reject:") else expect
    row["reference_expect"] = reference_expect
    first, second = compile_native(case_dir, src_file.name), compile_native(case_dir, src_file.name)
    row["compiler_run"] = first
    row["checks"]["deterministic"] = first["line"] == second["line"] and first["rc"] == second["rc"]
    row["checks"]["compiler_completed"] = first["rc"] == 0 and first["outcome"] == "ok"
    line = first["line"]
    native = {}
    if line.startswith("hex "):
        hexf = OUT / "native" / f"{name}.hex"
        hexf.write_text(line[4:])
        art = OUT / "native" / f"{name}.smc"
        hrc, _, hse = run([sys.executable, HERE / "hex2bin.py", hexf, art])
        row["checks"]["transport"] = hrc == 0
        native = {"smc_sha256": sha(art.read_bytes()), "bytes": art.stat().st_size,
                  "c0": execute(C0, art), "dev": execute(SMC, art)}
        _, dis, _ = run([SMC, "disasm", art])
        (OUT / "native" / f"{name}.disasm.txt").write_text(dis)
    else:
        m = re.match(r"error (\w+) @(\d+): (.*)", line)
        native = {"rejected": {"cat": m.group(1), "msg": m.group(3)} if m else {"cat": None, "msg": line}}
    row["native"] = native
    reference = {}
    for label, smc in (("c0", C0), ("dev", SMC)):
        art = OUT / "reference" / f"{name}.{label}.smc"
        art.unlink(missing_ok=True)
        crc, cso, cse = run([smc, "compile", src_file, "-o", art, "--opt-level", "O0"])
        if crc != 0:
            reference[label] = {"kind": "compile_error", "compile_rc": crc, "artifact_exists": art.exists(), "detail": (cso + cse).strip()}
        else:
            reference[label] = {"compile_rc": crc, "smc_sha256": sha(art.read_bytes()), **execute(smc, art)}
        row["checks"][f"reference_{label}_O0_expected"] = reference_matches(reference_expect, reference[label])
    row["reference"] = reference
    optimized_reference = {}
    for label, smc in (("c0", C0), ("dev", SMC)):
        art = OUT / "reference" / f"{name}.{label}.O1.smc"
        art.unlink(missing_ok=True)
        crc, cso, cse = run([smc, "compile", src_file, "-o", art, "--opt-level", "O1"])
        optimized_reference[label] = ({"kind": "compile_error", "compile_rc": crc, "artifact_exists": art.exists(), "detail": (cso + cse).strip()}
                                      if crc else {"compile_rc": crc, "smc_sha256": sha(art.read_bytes()), **execute(smc, art)})
        row["checks"][f"reference_{label}_O1_expected"] = reference_matches(reference_expect, optimized_reference[label])
        row["checks"][f"reference_{label}_O0_O1_parity"] = reference_pair_matches(reference_expect, reference[label], optimized_reference[label])
    row["reference_O1"] = optimized_reference
    if expect.startswith("reject:"):
        _, cat, frag = expect.split(":", 2)
        rej = native.get("rejected")
        row["checks"]["expected"] = bool(rej) and rej["cat"] == cat and frag in rej["msg"]
        # The reference compiler is recorded, not required to agree: a construct outside the
        # subset can be valid Semantic.
        row["reference_rejects"] = all(r["kind"] == "compile_error" for r in reference.values())
    else:
        kind = "ok" if expect == "ok" else "trap"
        frag = expect.split(":", 1)[1] if kind == "trap" else ""
        for vm in ("c0", "dev"):
            o = native.get(vm)
            row["checks"][f"expected_on_{vm}_vm"] = bool(o) and o["kind"] == kind and frag in o.get("detail", "")
        for ref in ("c0", "dev"):
            r = reference[ref]
            row["checks"][f"matches_{ref}_reference"] = r["kind"] in ("ok", "trap") and native.get(ref) is not None \
                and outcome_key(native[ref]) == outcome_key(r)
    row["pass"] = all(row["checks"].values())
    if not row["pass"]:
        failures.append(name)
    c = first["counters"] or {}
    bad = " ".join(k for k, v in row["checks"].items() if not v)
    print(f"{'PASS' if row['pass'] else 'FAIL'} {name:<46} steps={c.get('steps')} calls={c.get('calls')} frames={c.get('peak_frames')} {bad}", flush=True)
    rows.append(row)

# ---------------------------------------------------------------- artifact mutations
mut_rows = []
base_path = OUT / "native" / "p02_two_functions.smc"
if base_path.exists():
    base = base_path.read_bytes()
    dis = (OUT / "native" / "p02_two_functions.disasm.txt").read_text()

    def patch(data, at, new):
        b = bytearray(data); b[at:at + len(new)] = new; return bytes(b)

    call_at = base.find(bytes([0x40, 0x01]))       # first CALL with a destination in the artifact
    mutations = [
        ("unmodified", base, True),
        ("truncated", base[:-1], False),
        ("bad_magic", patch(base, 0, b"SEMCODEX"), False),
        ("unsupported_revision", patch(base, 0, b"SEMCOD99"), False),
        ("adt0_count_corrupted", patch(base, 12, struct.pack("<H", 9)), False),
        ("call_argc_changed", patch(base, call_at + 6, struct.pack("<H", 1)) if call_at > 0 else base, False),
        ("call_target_symbol_out_of_range", patch(base, call_at + 4, struct.pack("<H", 99)) if call_at > 0 else base, False),
        ("sig0_tag_corrupted", base.replace(b"SIG0", b"SIGX", 1), False),
        ("own0_tag_corrupted", base.replace(b"OWN0", b"OWNX", 1), False),
    ]
    for mname, data, should_pass in mutations:
        f = OUT / "mutations" / f"{mname}.smc"
        f.write_bytes(data)
        res = {}
        ok = True
        for label, smc in (("c0", C0), ("dev", SMC)):
            vrc, vso, vse = run([smc, "verify", f])
            res[label] = {"rc": vrc, "msg": (vso + vse).strip()[-200:]}
            ok = ok and ((vrc == 0) == should_pass)
        mut_rows.append({"mutation": mname, "expect": "accept" if should_pass else "reject", "verify": res, "pass": ok})
        if not ok:
            failures.append(f"mutation:{mname}")
        print(f"{'PASS' if ok else 'FAIL'} mutation:{mname:<38} c0_rc={res['c0']['rc']} dev_rc={res['dev']['rc']} {res['dev']['msg'][-110:]}", flush=True)
else:
    failures.append("mutation:base_artifact_missing")

summary = {
    "runner": "tests/compiler/run_tests.py",
    "reference_identity": REFERENCE_IDENTITY,
    "compiler_source_sha256": {p.name: sha(p.read_bytes()) for p in sorted(COMPILER.glob("*.sm"))},
    "smc_running_compiler": {"path_sha256": sha(SMC.read_bytes()), "version": run([SMC, "version", "--json"])[1]},
    "c0_smc": {"path_sha256": sha(C0.read_bytes()), "version": run([C0, "version", "--json"])[1]},
    "compiler_envelope": {"envelope": "trusted-compiler", "max_steps": int(MAX_STEPS), "max_calls": 50000, "max_frames": 256},
    "compiler_artifacts": {opt: sha(path.read_bytes()) for opt, path in COMPILER_ARTIFACTS.items()},
    "compiler_execution_limitation": "Pinned smc run uses its fixed source compiler path. O0/O1 compiler artifacts are compiled/verified; only test-program reference artifacts are executed at both optimization levels. run-smc does not expose application capabilities/quotas.",
    "cases": len(rows), "passed": len(rows) - len([r for r in rows if not r["pass"]]),
    "failures": failures, "rows": rows, "mutations": mut_rows,
}
(OUT / "results.json").write_text(json.dumps(summary, indent=1))
print(f"\n{summary['passed']}/{summary['cases']} cases passed; {len(mut_rows)} mutations; failures: {failures}")
print(f"evidence: {OUT / 'results.json'}")
sys.exit(1 if failures else 0)
