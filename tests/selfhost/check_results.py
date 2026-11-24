#!/usr/bin/env python3
"""Fail-closed result checker for a tests/selfhost attempt directory.

  python -I tests/selfhost/check_results.py <attempt> [--write] [--partial]
  python -I tests/selfhost/check_results.py --selftest <passing attempt> <new scratch dir>

The checker never trusts a recorded verdict and is deliberately independent of selfhost.py
(it re-implements the framing, the classification, the import expectations and the mutation
table). From the raw files of the attempt it recomputes:
  * the exact file inventory and every command-log hash;
  * which tool ran each command (argv) and that each tool still hashes to its pin, if present;
  * the C0 checkout attestation from the logged git output;
  * the source-set identity of the snapshot;
  * every generation: decoded again from the producing run's stdout and bound to the executor
    provenance of the previous generation; under S2, the accepted record and published hash;
  * the byte-equality comparisons;
  * the corpus: it must equal the repository's tests/compiler/cases.json plus the pinned import
    fixtures, with identical fixture bytes; every fixture run is bound to its generation;
    verdicts are recomputed from stdout, stderr provenance and VM logs (both VMs for traps);
  * the two pinned compiler-source mutations, re-applied to the snapshot.
A zero exit code alone never passes anything. --partial accepts an attempt without the
differential phase (scope "partial", e.g. the historical reproduction); the verdict says so.
--selftest copies a passing attempt, injects known violations one at a time and requires every
one to be rejected while the unmodified copy passes.
"""
import hashlib
import json
import pathlib
import re
import shutil
import sys

sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
PINS = json.loads((HERE / "pins.json").read_text(encoding="utf-8"))
OUTPUTS = ("report.json", "summary.md")
IMPORT_EXPECT = {
    "positive": "ok",
    "cycle": "reject:unsupported:cyclic executable helper import detected",
    "duplicate": "reject:name:function 'score' is defined more than once",
    "escape": "reject:unsupported:import path escapes the declared source root",
    "unsupported": "reject:unsupported:unsupported import form",
    "missing": "fault:ABI FsRead failure",
}
MUTATIONS = {
    "comment_only": ("lower.sm", None, "\n// selfhost mutation probe: comment only\n", "same_artifact"),
    "diagnostic_text": ("parser.sm", "expected '{'", "expected '{' [mutated]", "different_artifact"),
}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def frame_identity(entries):
    """shf0-source-set-v1 framing, reimplemented here (not imported) so the checker is independent."""
    h = hashlib.sha256(b"SHF0-SOURCE-SET\x00v1\x00")
    h.update(len(entries).to_bytes(8, "big"))
    for p, c in entries:
        pb = p.encode("utf-8")
        h.update(len(pb).to_bytes(4, "big") + pb + len(c).to_bytes(8, "big") + c)
    return "sha256:" + h.hexdigest()


def classify(out):
    if out.startswith(b"hex "):
        body = out[4:].strip()
        if len(body) % 2 or not re.fullmatch(rb"[0-9a-f]*", body):
            return {"kind": "malformed_hex"}
        return {"kind": "semcode", "semcode": bytes.fromhex(body.decode("ascii"))}
    m = re.fullmatch(rb"error (\w+) @(\d+): (.*?)\s*", out, re.S)
    if m:
        return {"kind": "diagnostic", "category": m.group(1).decode(), "message": m.group(3).decode("utf-8", "replace")}
    return {"kind": "no_output" if not out.strip() else "other"}


def provenance(stderr):
    for line in reversed(stderr.decode("utf-8", "replace").splitlines()):
        if line.startswith("{") and "stored_exec_provenance" in line:
            try:
                return json.loads(line)
            except ValueError:
                continue
    return None


def norm(p):
    return str(p).replace("\\", "/").lower()


def mutated_sources(snapshot, name):
    fname, old, new, _ = MUTATIONS[name]
    out = {}
    for f, data in snapshot.items():
        if f == fname:
            text = data.decode("utf-8")
            text = text + new if old is None else text.replace(old, new, 1)
            data = text.encode("utf-8")
        out[f] = data
    return out


class Check:
    def __init__(self, root):
        self.root = pathlib.Path(root).resolve()
        self.violations = []
        self.facts = {}
        self.expected = set()
        self.allowed_dirs = []

    def bad(self, msg):
        self.violations.append(msg)

    def read(self, rel):
        self.expected.add(rel)
        p = self.root / rel
        if not p.is_file():
            self.bad(f"missing file {rel}")
            return None
        return p.read_bytes()

    def json(self, rel, required=True):
        if not required and not (self.root / rel).is_file():
            return None
        b = self.read(rel)
        if b is None:
            return None
        try:
            return json.loads(b)
        except ValueError:
            self.bad(f"{rel} is not JSON")
            return None


def sidecar(c, rel, data):
    """smc compile writes <artifact>.provenance.json; it must name exactly the artifact beside it."""
    b = c.read(rel + ".provenance.json")
    if b is not None and data is not None:
        try:
            if json.loads(b).get("artifact_hash") != "sha256:" + sha(data):
                c.bad(f"{rel}: compiler provenance sidecar names a different artifact")
        except ValueError:
            c.bad(f"{rel}: compiler provenance sidecar is not JSON")


def check(root, partial=False):
    c = Check(root)
    cmds = {}
    raw = c.read("commands.jsonl")
    for line in (raw or b"").decode("utf-8").splitlines():
        e = json.loads(line)
        if e["tag"] in cmds:
            c.bad(f"duplicate command tag {e['tag']}")
        cmds[e["tag"]] = e
        for s in ("stdout", "stderr"):
            b = c.read(f"logs/{e['tag']}.{s}")
            if b is not None and sha(b) != e[f"{s}_sha256"]:
                c.bad(f"log logs/{e['tag']}.{s} does not match its recorded hash")

    def log(tag, stream="stdout"):
        if tag not in cmds:
            c.bad(f"no command {tag}")
            return b""
        p = c.root / "logs" / f"{tag}.{stream}"
        return p.read_bytes() if p.is_file() else b""

    def exit_of(tag):
        return cmds[tag]["exit"] if tag in cmds else None

    pre = c.json("preflight.json") or {}
    tools = pre.get("tools") or {}

    def argv_is(tag, tool, sub=None, path=None):
        """Binds a command to the tool that ran it, its subcommand and its target file."""
        a = (cmds.get(tag) or {}).get("argv") or []
        want_tool = (tools.get(tool) or {}).get("path")
        ok = bool(a) and want_tool is not None and norm(a[0]) == norm(want_tool)
        if sub is not None:
            ok = ok and len(a) > 1 and a[1] == sub
        if path is not None:
            i = 2 if sub is not None else 1
            ok = ok and len(a) > i and norm(a[i]).endswith("/" + path.lower())
        if not ok:
            c.bad(f"command {tag} did not run {tool} {sub or ''} {path or ''}".rstrip())

    # ---------------------------------------------------------------- preflight
    for name in ("smc", "driver"):
        t = tools.get(name) or {}
        if t.get("sha256") != PINS["tools"][name]["sha256"]:
            c.bad(f"preflight: {name} identity is not the pinned identity")
        p = pathlib.Path(t.get("path") or "")
        if t.get("path") and p.is_file():
            if sha(p.read_bytes()) != PINS["tools"][name]["sha256"]:
                c.bad(f"preflight: {name} at {p} no longer hashes to its pin")
            c.facts.setdefault("tools_rechecked", []).append(name)
    c0 = pre.get("c0_checkout") or {}
    head_log = log("preflight.c0_head").decode("utf-8", "replace").strip()
    if (exit_of("preflight.c0_head") != 0 or head_log != PINS["c0"]["sha"] or c0.get("head") != head_log
            or exit_of("preflight.c0_status") != 0 or log("preflight.c0_status").strip() != b"" or c0.get("clean") is not True
            or c0.get("verify_reference_checkout_errors")):
        c.bad("preflight: C0 checkout attestation does not hold (logged head/status, reference check)")
    files = PINS["source_set"]["files"]
    snapshot = {}
    for f in files:
        snapshot[f.split("/", 1)[1]] = c.read("snapshot/" + f.split("/", 1)[1]) or b""
    entries = [(f, snapshot[f.split("/", 1)[1]]) for f in files]
    snap_id = frame_identity(entries)
    src = pre.get("source_set") or {}
    if src.get("identity") != snap_id or src.get("snapshot_identity") != snap_id:
        c.bad("preflight: recorded source-set identity does not match the snapshot")
    if [r.get("path") for r in src.get("files", [])] != files:
        c.bad("preflight: source-set file list differs from pins.json")
    for (f, b), r in zip(entries, src.get("files", [])):
        if r.get("sha256") != sha(b):
            c.bad(f"preflight: snapshot file {f} differs from the recorded source")
    s2_configured = "s2" in tools
    if s2_configured:
        if exit_of("preflight.s2_identity") != 0:
            c.bad("preflight: s2 --print-identity failed")
        argv_is("preflight.s2_identity", "s2")
    if (pre.get("status") == "pass") == bool(c.violations):
        c.bad("preflight: recorded status disagrees with the evidence")
    c.facts["source_set_identity"] = snap_id
    repo_entries = [(f, (REPO / f).read_bytes() if (REPO / f).is_file() else b"") for f in files]
    c.facts["snapshot_is_repository_compiler"] = frame_identity(repo_entries) == snap_id

    # ---------------------------------------------------------------- build
    b = c.json("build.json") or {}
    gens = {}
    n_before = len(c.violations)
    if b.get("source_set_identity") != snap_id or b.get("snapshot_identity_after") != snap_id:
        c.bad("build: source-set identity changed during the build")
    prev = None
    for i, g in enumerate(b.get("generations", [])):
        name = g.get("name")
        if name != f"C{i + 1}":
            c.bad(f"build: generation order broken at {name}")
        rel = f"gen/{name}.smc"
        if g.get("artifact") != rel:
            c.bad(f"build: {name} artifact path is not {rel}")
        art = c.read(rel)
        if art is None:
            continue
        if sha(art) != g.get("sha256"):
            c.bad(f"build: {name} artifact hash differs from the record")
        if exit_of(f"build.{name}.verify") != 0:
            c.bad(f"build: {name} was not admitted by sm-verify")
        argv_is(f"build.{name}.verify", "smc", "verify", rel)
        if name == "C1":
            sidecar(c, rel, art)
            argv_is("build.C1.compile", "smc", "compile", "snapshot/" + PINS["source_set"]["entry"])
            if exit_of("build.C1.compile") != 0:
                c.bad("build: C0 compilation of S failed")
        else:
            ex = g.get("execution") or {}
            tag = f"build.{name}.execute"
            if ex.get("executor") != ("s2" if s2_configured else "driver"):
                c.bad(f"build: {name} executor does not match the declared configuration")
            if s2_configured:
                d = f"s2/{tag}"
                c.allowed_dirs.append(d + "/")
                argv_is(tag, "s2")
                argv_is(tag + ".verify_attempt", "s2", "--verify-attempt")
                if exit_of(tag) != 0 or exit_of(tag + ".verify_attempt") != 0:
                    c.bad(f"build: {name}: S2 did not commit and accept the attempt")
                rec = c.json(f"{d}/s2_record.json") or {}
                acc = c.read(f"{d}/accepted/{name}.smc")
                pub = rec.get("published") or {}
                if rec.get("accepted") is not True or acc != art or pub.get("sha256") != sha(art):
                    c.bad(f"build: {name}: the S2 record does not publish exactly the generation artifact")
                stderr = c.read(f"{d}/capture/stderr.bin") or b""
                stdout = c.read(f"{d}/capture/stdout.bin") or b""
                c.facts.setdefault("s2", {})[name] = {"containment_verdict": (rec.get("containment") or {}).get("verdict"),
                                                       "harness": (rec.get("harness") or {}).get("sha256") if isinstance(rec.get("harness"), dict) else rec.get("harness")}
            else:
                argv_is(tag, "driver", None, f"gen/{prev.get('name') if prev else '?'}.smc")
                stderr, stdout = log(tag, "stderr"), log(tag)
                if exit_of(tag) != 0:
                    c.bad(f"build: {name}: driver exit code is not 0")
            cl = classify(stdout)
            if cl.get("semcode") != art:
                c.bad(f"build: {name}: artifact is not the decoded stdout of the producing run")
            pv = provenance(stderr) or {}
            if pv.get("outcome") != "executed_ok" or pv.get("executed_artifact_sha256") != (prev or {}).get("sha256"):
                c.bad(f"build: {name}: executor provenance does not bind the previous generation")
            c.facts.setdefault("counters", {})[name] = pv.get("counters")
        gens[name] = {"sha256": sha(art), "bytes": len(art), "data": art}
        prev = g
    if len(gens) < 2:
        c.bad("build: fewer than two generations")
    if (b.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("build: recorded status disagrees with the evidence")

    # ---------------------------------------------------------------- verify
    v = c.json("verify.json") or {}
    n_before = len(c.violations)
    computed = {}
    for a, d in (("C1", "C2"), ("C2", "C3")):
        if a in gens and d in gens:
            computed[(a, d)] = gens[a]["data"] == gens[d]["data"]
    for r in v.get("comparisons", []):
        k = (r.get("a"), r.get("b"))
        if k not in computed or r.get("equal") != computed[k]:
            c.bad(f"verify: comparison {k} disagrees with the artifact bytes")
    if len(v.get("comparisons", [])) != len(computed):
        c.bad("verify: comparisons are incomplete")
    rule = v.get("expect")
    want = {"c1c2": [("C1", "C2"), ("C2", "C3")], "c2c3": [("C2", "C3")]}.get(rule)
    if want is None:
        c.bad("verify: unknown fixed-point expectation")
        want = []
    if not all(computed.get(k) for k in want):
        c.bad(f"verify: the declared fixed point ({rule}) does not hold")
    if (v.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("verify: recorded status disagrees with the evidence")
    c.facts["fixed_point_rule"] = rule
    c.facts["c1_equals_c2"] = computed.get(("C1", "C2"))
    c.facts["c2_equals_c3"] = computed.get(("C2", "C3"))
    c.facts["generations"] = {k: {"sha256": g["sha256"], "bytes": g["bytes"]} for k, g in gens.items()}

    # ---------------------------------------------------------------- differential
    d = c.json("differential.json", required=not partial)
    c.facts["scope"] = "full" if d is not None else "partial"
    if d is not None:
        check_differential(c, d, gens, snapshot, snap_id, log, exit_of, argv_is)

    # ---------------------------------------------------------------- inventory
    for tag in cmds:
        c.expected.update({f"logs/{tag}.stdout", f"logs/{tag}.stderr"})
    present = set()
    for p in c.root.rglob("*"):
        if p.is_symlink() or (p.is_dir() and getattr(p, "is_junction", lambda: False)()):
            c.bad(f"link in attempt: {p.relative_to(c.root).as_posix()}")
        if p.is_file():
            present.add(p.relative_to(c.root).as_posix())
    allowed = {x for x in present if any(x.startswith(a) for a in c.allowed_dirs)}
    extra = present - c.expected - set(OUTPUTS) - allowed - {"preflight.json", "build.json", "verify.json", "differential.json"}
    if extra:
        c.bad(f"inventory: {len(extra)} unexpected file(s): {sorted(extra)[:5]}")
    missing = {x for x in c.expected if x not in present}
    if missing:
        c.bad(f"inventory: {len(missing)} missing file(s): {sorted(missing)[:5]}")
    return c


def check_differential(c, d, gens, snapshot, snap_id, log, exit_of, argv_is):
    n_before = len(c.violations)
    repo_cases = json.loads((REPO / "tests/compiler/cases.json").read_text(encoding="utf-8"))
    packaged = c.read("corpus/cases.json")
    if packaged is None or json.loads(packaged) != repo_cases:
        c.bad("differential: packaged corpus differs from the repository's tests/compiler/cases.json")
    expected = {f"case.{n}": (e, "tests/compiler/cases", [f"{n}.sm"]) for n, e in repo_cases.items()}
    for n, e in IMPORT_EXPECT.items():
        dd = REPO / "tests/compiler/imports" / n
        expected[f"import.{n}"] = (e, f"tests/compiler/imports/{n}", sorted(p.name for p in dd.iterdir() if p.suffix == ".sm"))
    rows = d.get("rows", [])
    ids = [r.get("id") for r in rows]
    if sorted(ids) != sorted(expected) or len(set(ids)) != len(ids):
        c.bad("differential: fixture inventory differs from the repository corpus (missing, extra or duplicate rows)")
    gl = d.get("generations") or []
    if not gl or any(g not in gens for g in gl):
        c.bad("differential: generations are not built artifacts")
        return
    artifacts = {}
    for r in rows:
        rid = r.get("id", "?")
        exp, src_dir, src_files = expected.get(rid, (None, None, []))
        if r.get("expect") != exp:
            c.bad(f"differential: {rid}: expectation differs from the repository corpus")
        if sorted((r.get("file_sha256") or {})) != src_files:
            c.bad(f"differential: {rid}: fixture file set differs from the repository")
        for f in src_files:
            fb = c.read(f"fixtures/{rid}/root/{f}")
            repo_bytes = (REPO / src_dir / f).read_bytes()
            if fb != repo_bytes or (r.get("file_sha256") or {}).get(f) != sha(repo_bytes):
                c.bad(f"differential: {rid}: fixture file {f} differs from the repository")
        entry = (r.get("c0") or {}).get("artifact")
        argv_is(f"diff.{rid}.c0", "smc", "compile", f"fixtures/{rid}/root/{src_files[0] if exp and rid.startswith('case.') else 'main.sm'}")
        ref = None
        if exit_of(f"diff.{rid}.c0") == 0:
            if entry != f"fixtures/{rid}/c0.smc":
                c.bad(f"differential: {rid}: C0 artifact missing")
            ref = c.read(f"fixtures/{rid}/c0.smc")
            if ref is not None:
                artifacts[sha(ref)] = f"fixtures/{rid}/c0.smc"
                sidecar(c, f"fixtures/{rid}/c0.smc", ref)
        elif entry:
            c.bad(f"differential: {rid}: C0 artifact recorded for a failed compilation")
        outs = {}
        for gname in gl + [gl[-1] + ".repeat"]:
            base = gname.split(".")[0]
            tag = f"diff.{rid}.{gname}"
            argv_is(tag, "driver", None, f"gen/{base}.smc")
            so = log(tag)
            cl = classify(so)
            cl["driver"] = provenance(log(tag, "stderr")) or {}
            cl["exit"] = exit_of(tag)
            if cl["driver"].get("executed_artifact_sha256") != gens[base]["sha256"]:
                c.bad(f"differential: {rid}/{gname}: the run is not bound to generation {base}")
            if cl["kind"] == "semcode":
                rel = f"fixtures/{rid}/{gname}.smc"
                if c.read(rel) != cl["semcode"]:
                    c.bad(f"differential: {rid}/{gname}: stored SemCode is not the compiler's stdout")
                artifacts[sha(cl["semcode"])] = rel
            rec = (r.get("gens") or {}).get(gname) or {}
            if rec.get("kind") != cl["kind"] or rec.get("stdout_sha256") != sha(so):
                c.bad(f"differential: {rid}/{gname}: recorded outcome differs from the raw stdout")
            outs[gname] = cl
        fails = recompute_row(exp, ref, outs, gl, exit_of, log)
        if bool(r.get("pass")) != (not fails):
            c.bad(f"differential: {rid}: recorded verdict {r.get('pass')} disagrees with the evidence {fails}")
        if fails:
            c.bad(f"differential: {rid}: fails: {fails}")
    execs = d.get("executions") or {}
    if sorted(execs) != sorted(artifacts):
        c.bad("differential: executed artifact set differs from the distinct SemCode artifacts")
    for h, rel in artifacts.items():
        argv_is(f"exec.{h[:16]}.verify", "smc", "verify", rel)
        if exit_of(f"exec.{h[:16]}.verify") == 0:
            argv_is(f"exec.{h[:16]}.run_smc", "smc", "run-smc", rel)
            argv_is(f"exec.{h[:16]}.driver_pure", "driver", None, rel)
    muts = {m.get("name"): m for m in (d.get("source_mutations") or [])}
    if sorted(muts) != sorted(MUTATIONS):
        c.bad("differential: source mutation set differs from the pinned mutations")
    for name, (_, _, _, expect) in MUTATIONS.items():
        m = muts.get(name) or {}
        want_src = mutated_sources(snapshot, name)
        msrc = {f: (c.read(f"mutations/{name}/src/{f}") or b"") for f in snapshot}
        if msrc != want_src:
            c.bad(f"differential: mutation {name}: sources are not exactly the pinned edit of the snapshot")
        mid = frame_identity([("compiler/" + f, msrc[f]) for f in snapshot])
        argv_is(f"mutation.{name}.compile", "smc", "compile", f"mutations/{name}/src/main.sm")
        art = c.read(f"mutations/{name}/C1.smc") if exit_of(f"mutation.{name}.compile") == 0 else None
        if art is not None:
            sidecar(c, f"mutations/{name}/C1.smc", art)
        same = art is not None and "C1" in gens and art == gens["C1"]["data"]
        ok = art is not None and mid != snap_id and same == (expect == "same_artifact")
        if m.get("expect") != expect or m.get("source_set_identity") != mid or bool(m.get("pass")) != ok or not ok:
            c.bad(f"differential: source mutation {name} is not proven")
    if (d.get("status") == "pass") != (len(c.violations) == n_before):
        c.bad("differential: recorded status disagrees with the evidence")
    c.facts["fixtures"] = len(rows)
    c.facts["distinct_artifacts"] = len(artifacts)


def recompute_row(expect, ref, outs, gl, exit_of, log):
    f = []
    if not gl or not expect:
        return ["no generations or expectation"]
    main = [outs.get(g, {}) for g in gl]
    rep = outs.get(gl[-1] + ".repeat", {})
    key = lambda o: (o.get("kind"), o.get("semcode"), o.get("category"), o.get("message"))  # noqa: E731
    if len({key(o) for o in main}) != 1:
        f.append("generations disagree")
    if key(rep) != key(main[-1]):
        f.append("nondeterministic")
    for o in main + [rep]:
        k = o.get("kind")
        drv = o.get("driver") or {}
        completed = o.get("exit") == 0 and drv.get("outcome") == "executed_ok"
        if expect == "ok" or expect.startswith("trap:"):
            if k != "semcode" or not completed:
                f.append("no SemCode from a completed compiler run")
                continue
            if ref is None or o["semcode"] != ref:
                f.append("SemCode differs from C0")
            h = sha(o["semcode"])[:16]
            if exit_of(f"exec.{h}.verify") != 0:
                f.append("not admitted")
                continue
            run_exit = exit_of(f"exec.{h}.run_smc")
            run_text = (log(f"exec.{h}.run_smc") + log(f"exec.{h}.run_smc", "stderr")).decode("utf-8", "replace")
            pure = provenance(log(f"exec.{h}.driver_pure", "stderr")) or {}
            if expect == "ok":
                if run_exit != 0 or exit_of(f"exec.{h}.driver_pure") != 0 or pure.get("outcome") != "executed_ok":
                    f.append("run failed")
            else:
                frag = expect.split(":", 1)[1]
                if run_exit in (0, None) or frag not in run_text:
                    f.append("no expected trap on smc run-smc")
                if pure.get("outcome") != "runtime_error" or frag not in (pure.get("error") or ""):
                    f.append("no expected trap on the driver")
        elif expect.startswith("reject:"):
            _, cat, frag = expect.split(":", 2)
            if k != "diagnostic" or o.get("category") != cat or frag not in o.get("message", "") or not completed:
                f.append("no expected diagnostic")
        elif expect.startswith("fault:"):
            frag = expect.split(":", 1)[1]
            if k == "semcode" or k == "diagnostic" or drv.get("outcome") != "runtime_error" or frag not in (drv.get("error") or ""):
                f.append("no expected process fault")
        else:
            f.append("unknown expectation")
    return sorted(set(f))


def write_outputs(c):
    ok = not c.violations
    f = c.facts
    success = ok and f.get("scope") == "full" and f.get("fixed_point_rule") == "c1c2" and f.get("c1_equals_c2") and f.get("c2_equals_c3")
    rep = {"schema": "semantic.selfhost.report/1", "attempt": str(c.root), "verdict": "PASS" if ok else "FAIL",
           "engineering_self_host_success": bool(success), "violations": c.violations, "facts": f}
    (c.root / "report.json").open("x", encoding="utf-8").write(json.dumps(rep, indent=1, sort_keys=True) + "\n")
    lines = [f"# Self-host attempt report: {rep['verdict']} (scope {f.get('scope')}, fixed-point rule {f.get('fixed_point_rule')})", "",
             f"- engineering self-host success (full scope, C1 == C2 == C3): {rep['engineering_self_host_success']}",
             f"- source-set identity: {f.get('source_set_identity')} (repository compiler: {f.get('snapshot_is_repository_compiler')})"]
    for k, g in (f.get("generations") or {}).items():
        lines.append(f"- {k}: sha256 {g['sha256']} ({g['bytes']} bytes); counters {(f.get('counters') or {}).get(k)}")
    lines += [f"- C1 == C2 (byte-equality-v1): {f.get('c1_equals_c2')}", f"- C2 == C3: {f.get('c2_equals_c3')}",
              f"- fixtures: {f.get('fixtures')}; distinct SemCode artifacts: {f.get('distinct_artifacts')}",
              f"- violations: {len(c.violations)}"] + [f"  - {v}" for v in c.violations[:50]]
    (c.root / "summary.md").open("x", encoding="utf-8").write("\n".join(lines) + "\n")


def mutate_json(path, fn):
    obj = json.loads(path.read_text(encoding="utf-8"))
    fn(obj)
    path.write_text(json.dumps(obj, indent=1, sort_keys=True) + "\n", encoding="utf-8")


def rewrite_log(root, tag, stream, data):
    (root / "logs" / f"{tag}.{stream}").write_bytes(data)
    out = []
    for line in (root / "commands.jsonl").read_text().splitlines():
        e = json.loads(line)
        if e["tag"] == tag:
            e[f"{stream}_sha256"] = sha(data)
        out.append(json.dumps(e, sort_keys=True))
    (root / "commands.jsonl").write_text("\n".join(out) + "\n")


def edit_command(root, tag, fn):
    out = []
    for line in (root / "commands.jsonl").read_text().splitlines():
        e = json.loads(line)
        if e["tag"] == tag:
            fn(e)
        out.append(json.dumps(e, sort_keys=True))
    (root / "commands.jsonl").write_text("\n".join(out) + "\n")


def selftest(src, scratch):
    src, scratch = pathlib.Path(src).resolve(), pathlib.Path(scratch).resolve()
    if scratch.exists():
        sys.exit("selftest: scratch directory must not exist")
    base = check(src)
    if base.violations:
        sys.exit(f"selftest: the source attempt does not pass ({len(base.violations)} violations)")
    diff = json.loads((src / "differential.json").read_text())
    first_ok = next(r for r in diff["rows"] if r["expect"] == "ok")["id"]
    trap_row = next(r for r in diff["rows"] if r["expect"].startswith("trap:"))
    trap_h = trap_row["c0"]["sha256"][:16]
    s2 = (src / "s2").is_dir()

    def flip_byte(p):
        b = bytearray(p.read_bytes()); b[len(b) // 2] ^= 1; p.write_bytes(bytes(b))

    def drop_case(r):
        mutate_json(r / "corpus" / "cases.json", lambda o: o.pop(first_ok[5:]))
        mutate_json(r / "differential.json", lambda o: o.__setitem__("rows", [x for x in o["rows"] if x["id"] != first_ok]))

    def wrong_generation(r):
        # the C2 run of a fixture actually executed C1 (argv and provenance both say so)
        tag = f"diff.{first_ok}.C2"
        c1 = json.loads((r / "build.json").read_text())["generations"][0]["sha256"]
        edit_command(r, tag, lambda e: e.__setitem__("argv", [e["argv"][0], e["argv"][1].replace("C2.smc", "C1.smc")] + e["argv"][2:]))
        se = (r / "logs" / f"{tag}.stderr").read_bytes().replace(b'"executed_artifact_sha256":"', b'"executed_artifact_sha256":"x', 1)
        rewrite_log(r, tag, "stderr", se)

    injections = {
        "artifact_byte_flip": lambda r: flip_byte(r / "gen" / "C2.smc"),
        "missing_log": lambda r: (r / "logs" / "build.C1.verify.stdout").unlink(),
        "extra_file": lambda r: (r / "gen" / "C9.smc").write_bytes(b"x"),
        "verify_claim_flip": lambda r: mutate_json(r / "verify.json", lambda o: o["comparisons"][0].__setitem__("equal", not o["comparisons"][0]["equal"])),
        "tool_identity": lambda r: mutate_json(r / "preflight.json", lambda o: o["tools"]["smc"].__setitem__("sha256", "0" * 64)),
        "c0_dirty_log": lambda r: rewrite_log(r, "preflight.c0_status", "stdout", b" M crates/sm-vm/src/lib.rs\n"),
        "verifier_rejects": lambda r: edit_command(r, "build.C2.verify", lambda e: e.__setitem__("exit", 1)),
        "verify_ran_other_command": lambda r: edit_command(r, "build.C2.verify", lambda e: e.__setitem__("argv", [e["argv"][0], "version"])),
        "exit_zero_but_diagnostic": lambda r: rewrite_log(r, f"diff.{first_ok}.C2", "stdout", b"error type @0: injected\n"),
        "fixture_run_wrong_generation": wrong_generation,
        "row_removed": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].pop()),
        "row_duplicated": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].append(o["rows"][0])),
        "corpus_case_dropped": drop_case,
        "expectation_rewritten": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("expect", "reject:type:x")),
        "import_expectation_rewritten": lambda r: mutate_json(r / "differential.json", lambda o: next(x for x in o["rows"] if x["id"] == "import.positive").__setitem__("expect", "reject:type:x")),
        "mutation_expectation_flipped": lambda r: mutate_json(r / "differential.json", lambda o: o["source_mutations"][0].__setitem__("expect", "different_artifact")),
        "mutation_source_altered": lambda r: (r / "mutations" / "diagnostic_text" / "src" / "lower.sm").write_bytes(b"// other\n" + (r / "mutations" / "diagnostic_text" / "src" / "lower.sm").read_bytes()),
        "trap_log_altered": lambda r: rewrite_log(r, f"exec.{trap_h}.run_smc", "stdout", b"runtime trap: SomethingElse\n"),
        "snapshot_mutated": lambda r: flip_byte(r / "snapshot" / "lexer.sm"),
        "status_forged": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("pass", False)),
        "vm_run_failed": lambda r: edit_command(r, next(e["tag"] for e in map(json.loads, (r / "commands.jsonl").read_text().splitlines()) if e["tag"].endswith(".run_smc")), lambda e: e.__setitem__("exit", 3)),
        "differential_deleted": lambda r: (r / "differential.json").unlink(),
    }
    if s2:
        injections["s2_record_not_accepted"] = lambda r: mutate_json(r / "s2" / "build.C2.execute" / "s2_record.json", lambda o: o.__setitem__("accepted", False))
        injections["s2_stray_file"] = lambda r: (r / "s2" / "stray.txt").write_bytes(b"x")
    results = {}
    scratch.mkdir()
    for name, inject in injections.items():
        d = scratch / name
        shutil.copytree(src, d, ignore=shutil.ignore_patterns(*OUTPUTS))
        inject(d)
        v = check(d).violations
        results[name] = {"detected": bool(v), "violations": v[:3]}
    clean = scratch / "unmodified"
    shutil.copytree(src, clean, ignore=shutil.ignore_patterns(*OUTPUTS))
    results["unmodified_copy"] = {"detected": bool(check(clean).violations)}
    ok = all(r["detected"] for k, r in results.items() if k != "unmodified_copy") and not results["unmodified_copy"]["detected"]
    out = {"schema": "semantic.selfhost.checker_selftest/1", "source": str(src), "injections": len(injections), "result": "PASS" if ok else "FAIL", "cases": results}
    (scratch / "selftest.json").open("x", encoding="utf-8").write(json.dumps(out, indent=1) + "\n")
    print(json.dumps({k: v["detected"] for k, v in results.items()}, indent=1))
    print(f"checker self-test: {out['result']} ({len(injections)} injections)")
    return 0 if ok else 1


def main(argv):
    if argv and argv[0] == "--selftest":
        return selftest(argv[1], argv[2])
    c = check(argv[0], partial="--partial" in argv)
    if "--write" in argv:
        write_outputs(c)
    print(f"check_results: {'PASS' if not c.violations else 'FAIL'} ({len(c.violations)} violations, scope {c.facts.get('scope')})")
    for v in c.violations[:20]:
        print("  - " + v)
    return 0 if not c.violations else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
