#!/usr/bin/env python3
"""Fail-closed qualification runner for the Semantic-written compiler (compiler/main.sm).

Environment (both required; no default binaries are guessed):
  SEMANTIC_SMC     smc-cli that RUNS the compiler. The compiler source needs text inspection and
                   u32 arithmetic, which the pinned C0 (v1.2.0) does not have, so this is an
                   exploratory reference build (record its SHA; results are not C0 qualification).
  SEMANTIC_C0_SMC  smc-cli built from the pinned C0 (reference/semantic-reference.toml). Used as
                   the reference compiler, verifier and VM for the compiled test programs, which
                   use no capability beyond C0.
Optional:
  SEMANTIC_TEST_OUT   output directory (default: $TMPDIR/semantic-compiler-tests); never inside
                      the source tree.
  COMPILER_MAX_STEPS  Steps budget for running the compiler (default 20000000, trusted-compiler
                      envelope; Calls 16384 and Frames 256 stay at their fixed values).

Per case:
  1. compile with the Semantic compiler twice -> deterministic result line required
  2. "hex ..." -> tests/compiler/hex2bin.py (transport only) -> .smc
  3. verify with BOTH verifiers, run on BOTH VMs (pure profile)
  4. compile with C0 and with SEMANTIC_SMC at O0, verify and run (reference behaviour)
  5. compare against cases.json and against the reference outcome
Mutations of generated artifacts must be rejected by both verifiers.
Exit status is nonzero if anything fails.
"""
import hashlib, json, os, pathlib, re, struct, subprocess, sys, tempfile

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
OUT = pathlib.Path(os.environ.get("SEMANTIC_TEST_OUT", pathlib.Path(tempfile.gettempdir()) / "semantic-compiler-tests"))
if REPO in OUT.resolve().parents or OUT.resolve() == REPO:
    sys.exit("run_tests: SEMANTIC_TEST_OUT must be outside the source tree")
MAX_STEPS = os.environ.get("COMPILER_MAX_STEPS", "20000000")
for sub in ("native", "reference", "mutations"):
    (OUT / sub).mkdir(parents=True, exist_ok=True)
sha = lambda b: hashlib.sha256(b).hexdigest()


def run(cmd, cwd=None):
    p = subprocess.run([str(c) for c in cmd], cwd=cwd, capture_output=True, text=True, timeout=1800)
    return p.returncode, p.stdout, p.stderr


def compile_native(case_dir, name):
    rc, so, se = run([SMC, "run", "main.sm", "--profile", "cli-read-only", "--root", case_dir,
                      "--envelope", "trusted-compiler", "--max-steps", MAX_STEPS, "--metrics", "json", "--", name], cwd=COMPILER)
    m = next((json.loads(l) for l in se.splitlines() if l.startswith('{"schema":"semantic.run.metrics')), {})
    return {"rc": rc, "line": so, "outcome": m.get("outcome"), "counters": m.get("counters"), "quota_exceeded": m.get("quota_exceeded")}


def execute(smc, art):
    vrc, vso, vse = run([smc, "verify", art])
    if vrc != 0:
        return {"kind": "rejected_by_verifier", "detail": (vso + vse).strip()}
    rrc, rso, rse = run([smc, "run-smc", art])
    msg = (rso + rse).strip()
    return {"kind": "ok" if rrc == 0 else "trap", "detail": msg}


def outcome_key(o):
    if o["kind"] == "trap":
        return ("trap", o["detail"].splitlines()[-1] if o["detail"] else "")
    return (o["kind"], "")


cases = json.loads((HERE / "cases.json").read_text())
case_dir = HERE / "cases"
rows, failures = [], []
for name, spec in sorted(cases.items()):
    expect = spec["expect"]
    c0_comparable = spec["c0_comparable"]
    src_file = case_dir / f"{name}.sm"
    row = {"case": name, "expect": expect, "c0_comparable": c0_comparable, "checks": {}}
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
            reference[label] = {"kind": "compile_error", "detail": (cso + cse).strip()}
        else:
            reference[label] = {"smc_sha256": sha(art.read_bytes()), **execute(smc, art)}
    row["reference"] = reference
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
        # Post-C0 cases are judged against the exploratory toolchain only; what C0 does
        # with them (expected: it cannot compile or admit them) is recorded, not judged.
        sides = ("c0", "dev") if c0_comparable else ("dev",)
        if not c0_comparable:
            row["c0_outcome_recorded"] = {"native_on_c0": native.get("c0"), "c0_reference": reference["c0"]}
        for vm in sides:
            o = native.get(vm)
            row["checks"][f"expected_on_{vm}_vm"] = bool(o) and o["kind"] == kind and frag in o.get("detail", "")
        for ref in sides:
            r = reference[ref]
            row["checks"][f"matches_{ref}_reference"] = r["kind"] != "compile_error" and native.get(ref) is not None \
                and outcome_key(native[ref]) == outcome_key(r)
    row["pass"] = all(row["checks"].values())
    if not row["pass"]:
        failures.append(name)
    c = first["counters"] or {}
    bad = " ".join(k for k, v in row["checks"].items() if not v)
    tag = "" if c0_comparable else " [post-C0: dev only]"
    print(f"{'PASS' if row['pass'] else 'FAIL'} {name:<46} steps={c.get('steps')} calls={c.get('calls')} frames={c.get('peak_frames')}{tag} {bad}", flush=True)
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
    "compiler_source_sha256": {p.name: sha(p.read_bytes()) for p in sorted(COMPILER.glob("*.sm"))},
    "smc_running_compiler": {"path_sha256": sha(SMC.read_bytes()), "version": run([SMC, "version", "--json"])[1]},
    "c0_smc": {"path_sha256": sha(C0.read_bytes()), "version": run([C0, "version", "--json"])[1]},
    "compiler_envelope": {"envelope": "trusted-compiler", "max_steps": int(MAX_STEPS)},
    "cases": len(rows), "passed": len(rows) - len([r for r in rows if not r["pass"]]),
    "failures": failures, "rows": rows, "mutations": mut_rows,
}
(OUT / "results.json").write_text(json.dumps(summary, indent=1))
print(f"\n{summary['passed']}/{summary['cases']} cases passed; {len(mut_rows)} mutations; failures: {failures}")
print(f"evidence: {OUT / 'results.json'}")
sys.exit(1 if failures else 0)
