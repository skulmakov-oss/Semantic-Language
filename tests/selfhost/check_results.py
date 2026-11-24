#!/usr/bin/env python3
"""Fail-closed result checker for a tests/selfhost attempt directory.

  python -I tests/selfhost/check_results.py <attempt> [--write]
  python -I tests/selfhost/check_results.py --selftest <passing attempt> <new scratch dir>

The checker never trusts a recorded verdict. It recomputes every claim from the raw files:
exact file inventory, command log hashes, tool and C0 identities against pins.json, the
source-set identity of the snapshot, every generation artifact (against the compiler's own
stdout bytes and the executor provenance), the byte-equality comparisons, and every fixture
verdict (classification of the saved stdout, SemCode identity against the C0 compilation,
verifier and VM exit codes, determinism, corpus completeness). A recorded 'pass' that the
raw evidence does not support is a violation. A zero exit code alone never passes anything.
--selftest copies a passing attempt, injects known violations one at a time and requires the
checker to reject every one of them (and to accept the unmodified copy).
"""
import hashlib
import importlib.util
import json
import pathlib
import re
import shutil
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent.parent
PINS = json.loads((HERE / "pins.json").read_text(encoding="utf-8"))
IMPORTS = ("cycle", "duplicate", "escape", "missing", "positive", "unsupported")
OUTPUTS = ("report.json", "summary.md")


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


class Check:
    def __init__(self, root):
        self.root = pathlib.Path(root).resolve()
        self.violations = []
        self.facts = {}
        self.expected = set()

    def bad(self, msg):
        self.violations.append(msg)

    def read(self, rel):
        self.expected.add(rel)
        p = self.root / rel
        if not p.is_file():
            self.bad(f"missing file {rel}")
            return None
        return p.read_bytes()

    def json(self, rel):
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


def check(root):
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

    # ---------------------------------------------------------------- preflight
    pre = c.json("preflight.json") or {}
    for name in ("smc", "driver"):
        t = (pre.get("tools") or {}).get(name) or {}
        if t.get("sha256") != PINS["tools"][name]["sha256"]:
            c.bad(f"preflight: {name} identity is not the pinned identity")
    c0 = pre.get("c0_checkout") or {}
    if c0.get("head") != PINS["c0"]["sha"] or c0.get("clean") is not True or c0.get("verify_reference_checkout_errors"):
        c.bad("preflight: C0 checkout attestation does not hold (head/clean/reference check)")
    files = PINS["source_set"]["files"]
    entries = []
    for f in files:
        b = c.read("snapshot/" + f.split("/", 1)[1])
        entries.append((f, b or b""))
    snap_id = frame_identity(entries)
    src = pre.get("source_set") or {}
    if src.get("identity") != snap_id or src.get("snapshot_identity") != snap_id:
        c.bad("preflight: recorded source-set identity does not match the snapshot")
    if [r.get("path") for r in src.get("files", [])] != files:
        c.bad("preflight: source-set file list differs from pins.json")
    for (f, b), r in zip(entries, src.get("files", [])):
        if r.get("sha256") != sha(b):
            c.bad(f"preflight: snapshot file {f} differs from the recorded source")
    if (tools_s2 := (pre.get("tools") or {}).get("s2")) is not None:
        if exit_of("preflight.s2_identity") != 0:
            c.bad("preflight: s2 --print-identity failed")
    pre_ok = not c.violations
    if (pre.get("status") == "pass") != pre_ok:
        c.bad("preflight: recorded status disagrees with the evidence")
    c.facts["source_set_identity"] = snap_id

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
        art = c.read(g.get("artifact") or f"gen/{name}.smc")
        if art is None:
            continue
        if sha(art) != g.get("sha256"):
            c.bad(f"build: {name} artifact hash differs from the record")
        if exit_of(f"build.{name}.verify") != 0:
            c.bad(f"build: {name} was not admitted by sm-verify")
        if name == "C1":
            sidecar(c, g.get("artifact") or "gen/C1.smc", art)
            if exit_of("build.C1.compile") != 0:
                c.bad("build: C0 compilation of S failed")
        else:
            ex = g.get("execution") or {}
            if ex.get("executor") == "s2":
                if exit_of(f"build.{name}.execute.verify_attempt") != 0:
                    c.bad(f"build: {name}: s2-harness --verify-attempt did not accept the attempt")
                acc = c.root / "s2" / f"build.{name}.execute" / "accepted" / f"{name}.smc"
                if not acc.is_file() or acc.read_bytes() != art:
                    c.bad(f"build: {name}: published S2 output differs from the generation artifact")
                cap = c.root / "s2" / f"build.{name}.execute" / "capture"
                stderr = (cap / "stderr.bin").read_bytes() if (cap / "stderr.bin").is_file() else b""
                stdout = (cap / "stdout.bin").read_bytes() if (cap / "stdout.bin").is_file() else b""
            else:
                stderr, stdout = log(f"build.{name}.execute", "stderr"), log(f"build.{name}.execute")
                if exit_of(f"build.{name}.execute") != 0:
                    c.bad(f"build: {name}: driver exit code is not 0")
            cl = classify(stdout)
            if cl.get("semcode") != art:
                c.bad(f"build: {name}: artifact is not the decoded stdout of the producing run")
            pv = provenance(stderr) or {}
            if pv.get("outcome") != "executed_ok" or pv.get("executed_artifact_sha256") != (prev or {}).get("sha256"):
                c.bad(f"build: {name}: executor provenance does not bind the previous generation")
        gens[name] = {"sha256": sha(art), "bytes": len(art), "data": art}
        prev = g
    if not gens:
        c.bad("build: no generation artifacts")
    if (b.get("status") == "pass") != (len(c.violations) == n_before and len(gens) >= 2):
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
    want = {"c1c2": [("C1", "C2"), ("C2", "C3")], "c2c3": [("C2", "C3")]}.get(v.get("expect"), None)
    if want is None:
        c.bad("verify: unknown fixed-point expectation")
        want = []
    v_ok = all(computed.get(k) for k in want) and len(c.violations) == n_before
    if (v.get("status") == "pass") != v_ok:
        c.bad("verify: recorded status disagrees with the evidence")
    c.facts["c1_equals_c2"] = computed.get(("C1", "C2"))
    c.facts["c2_equals_c3"] = computed.get(("C2", "C3"))
    c.facts["generations"] = {k: {"sha256": g["sha256"], "bytes": g["bytes"]} for k, g in gens.items()}

    # ---------------------------------------------------------------- differential
    d = c.json("differential.json")
    if d is not None:
        n_before = len(c.violations)
        cases_raw = c.read("corpus/cases.json") or b"{}"
        cases = json.loads(cases_raw)
        want_ids = sorted([f"case.{n}" for n in cases] + [f"import.{n}" for n in IMPORTS])
        rows = d.get("rows", [])
        ids = [r.get("id") for r in rows]
        if sorted(ids) != want_ids or len(set(ids)) != len(ids):
            c.bad("differential: fixture inventory differs from the corpus (missing, extra or duplicate rows)")
        gl = d.get("generations") or []
        if not gl or any(g not in gens for g in gl):
            c.bad("differential: generations are not built artifacts")
        artifacts = {}
        for r in rows:
            rid = r.get("id", "?")
            exp = cases.get(rid[5:]) if rid.startswith("case.") else None
            if rid.startswith("case.") and r.get("expect") != exp:
                c.bad(f"differential: {rid}: expectation differs from the corpus")
            if rid.startswith("import.") and r.get("expect") is None:
                c.bad(f"differential: {rid}: missing expectation")
            for f, h in (r.get("file_sha256") or {}).items():
                fb = c.read(f"fixtures/{rid}/root/{f}")
                if fb is not None and sha(fb) != h:
                    c.bad(f"differential: {rid}: fixture file {f} differs from the record")
            ref = c.read(r["c0"]["artifact"]) if (r.get("c0") or {}).get("artifact") else None
            if ref is not None:
                artifacts[sha(ref)] = r["c0"]["artifact"]
                sidecar(c, r["c0"]["artifact"], ref)
            outs = {}
            for gname in gl + [gl[-1] + ".repeat"] if gl else []:
                tag = f"diff.{rid}.{gname}"
                so = log(tag)
                cl = classify(so)
                cl["driver"] = provenance(log(tag, "stderr")) or {}
                if cl["kind"] == "semcode":
                    rel = f"fixtures/{rid}/{gname}.smc"
                    ab = c.read(rel)
                    if ab != cl["semcode"]:
                        c.bad(f"differential: {rid}/{gname}: stored SemCode is not the compiler's stdout")
                    artifacts[sha(cl["semcode"])] = rel
                rec = (r.get("gens") or {}).get(gname) or {}
                if rec.get("kind") != cl["kind"] or rec.get("stdout_sha256") != sha(so):
                    c.bad(f"differential: {rid}/{gname}: recorded outcome differs from the raw stdout")
                outs[gname] = cl
            fails = recompute_row(r.get("expect"), ref, outs, gl, d.get("executions") or {}, exit_of, cmds, log)
            if bool(r.get("pass")) != (not fails):
                c.bad(f"differential: {rid}: recorded verdict {r.get('pass')} disagrees with the evidence {fails}")
            if fails:
                c.bad(f"differential: {rid}: fails: {fails}")
        execs = d.get("executions") or {}
        if sorted(execs) != sorted(artifacts):
            c.bad("differential: executed artifact set differs from the distinct SemCode artifacts")
        muts = d.get("source_mutations") or []
        if sorted(m.get("name") for m in muts) != ["comment_only", "diagnostic_text"]:
            c.bad("differential: source mutation set incomplete")
        for m in muts:
            art = c.read(f"mutations/{m.get('name')}/C1.smc")
            sidecar(c, f"mutations/{m.get('name')}/C1.smc", art)
            msrc = [(f, (c.read(f"mutations/{m.get('name')}/src/{f.split('/', 1)[1]}") or b"")) for f in files]
            mid = frame_identity(msrc)
            same = art is not None and gens.get("C1") and art == gens["C1"]["data"]
            ok = exit_of(f"mutation.{m.get('name')}.compile") == 0 and mid != snap_id and art is not None and same == (m.get("expect") == "same_artifact")
            if m.get("source_set_identity") != mid or bool(m.get("pass")) != ok or not ok:
                c.bad(f"differential: source mutation {m.get('name')} is not proven")
        if (d.get("status") == "pass") != (len(c.violations) == n_before):
            c.bad("differential: recorded status disagrees with the evidence")
        c.facts["fixtures"] = len(rows)
        c.facts["distinct_artifacts"] = len(artifacts)

    # ---------------------------------------------------------------- inventory
    for tag in cmds:
        c.expected.update({f"logs/{tag}.stdout", f"logs/{tag}.stderr"})
    present = set()
    for p in c.root.rglob("*"):
        if p.is_symlink() or (p.is_dir() and getattr(p, "is_junction", lambda: False)()):
            c.bad(f"link in attempt: {p.relative_to(c.root).as_posix()}")
        if p.is_file():
            present.add(p.relative_to(c.root).as_posix())
    s2_files = {x for x in present if x.startswith("s2/")}
    extra = present - c.expected - set(OUTPUTS) - s2_files - {"preflight.json", "build.json", "verify.json", "differential.json"}
    if extra:
        c.bad(f"inventory: {len(extra)} unexpected file(s): {sorted(extra)[:5]}")
    missing = {x for x in c.expected if x not in present}
    if missing:
        c.bad(f"inventory: {len(missing)} missing file(s): {sorted(missing)[:5]}")
    return c


def recompute_row(expect, ref, outs, gl, execs, exit_of, cmds, cmd_log=lambda tag, stream: b""):
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
    for o in main:
        k = o.get("kind")
        d = o.get("driver") or {}
        if expect == "ok" or expect.startswith("trap:"):
            if k != "semcode":
                f.append("no SemCode")
                continue
            if ref is None or o["semcode"] != ref:
                f.append("SemCode differs from C0")
            h = sha(o["semcode"])[:16]
            if exit_of(f"exec.{h}.verify") != 0:
                f.append("not admitted")
                continue
            run_exit = exit_of(f"exec.{h}.run_smc")
            if expect == "ok":
                pure = provenance(cmd_log(f"exec.{h}.driver_pure", "stderr")) or {}
                if run_exit != 0 or exit_of(f"exec.{h}.driver_pure") != 0 or pure.get("outcome") != "executed_ok":
                    f.append("run failed")
            else:
                frag = expect.split(":", 1)[1]
                e = execs.get(sha(o["semcode"]), {})
                if run_exit in (0, None) or frag not in e.get("run_output", ""):
                    f.append("no expected trap")
        elif expect.startswith("reject:"):
            _, cat, frag = expect.split(":", 2)
            if k != "diagnostic" or o.get("category") != cat or frag not in o.get("message", "") or d.get("outcome") != "executed_ok":
                f.append("no expected diagnostic")
        elif expect.startswith("fault:"):
            frag = expect.split(":", 1)[1]
            if k == "semcode" or d.get("outcome") == "executed_ok" or frag not in (d.get("error") or ""):
                f.append("no expected process fault")
        else:
            f.append("unknown expectation")
    return f


def write_outputs(c):
    ok = not c.violations
    rep = {"schema": "semantic.selfhost.report/1", "attempt": str(c.root), "verdict": "PASS" if ok else "FAIL",
           "violations": c.violations, "facts": c.facts}
    (c.root / "report.json").open("x", encoding="utf-8").write(json.dumps(rep, indent=1, sort_keys=True) + "\n")
    lines = [f"# Self-host attempt report: {rep['verdict']}", "", f"- source-set identity: {c.facts.get('source_set_identity')}"]
    for k, g in (c.facts.get("generations") or {}).items():
        lines.append(f"- {k}: sha256 {g['sha256']} ({g['bytes']} bytes)")
    lines += [f"- C1 == C2 (byte-equality-v1): {c.facts.get('c1_equals_c2')}", f"- C2 == C3: {c.facts.get('c2_equals_c3')}",
              f"- fixtures: {c.facts.get('fixtures')}; distinct SemCode artifacts: {c.facts.get('distinct_artifacts')}",
              f"- violations: {len(c.violations)}"] + [f"  - {v}" for v in c.violations[:50]]
    (c.root / "summary.md").open("x", encoding="utf-8").write("\n".join(lines) + "\n")


def mutate_json(path, fn):
    obj = json.loads(path.read_text(encoding="utf-8"))
    fn(obj)
    path.write_text(json.dumps(obj, indent=1, sort_keys=True) + "\n", encoding="utf-8")


def selftest(src, scratch):
    src, scratch = pathlib.Path(src).resolve(), pathlib.Path(scratch).resolve()
    if scratch.exists():
        sys.exit("selftest: scratch directory must not exist")
    base = check(src)
    if base.violations:
        sys.exit(f"selftest: the source attempt does not pass ({len(base.violations)} violations)")
    first_ok = next(r for r in json.loads((src / "differential.json").read_text())["rows"] if r["expect"] == "ok")["id"]

    def flip_byte(p):
        b = bytearray(p.read_bytes()); b[len(b) // 2] ^= 1; p.write_bytes(bytes(b))

    def fake_diag(root):
        tag = f"diff.{first_ok}.C2"
        new = b"error type @0: injected\n"
        (root / "logs" / f"{tag}.stdout").write_bytes(new)
        lines = (root / "commands.jsonl").read_text().splitlines()
        out = []
        for line in lines:
            e = json.loads(line)
            if e["tag"] == tag:
                e["stdout_sha256"] = sha(new)
            out.append(json.dumps(e, sort_keys=True))
        (root / "commands.jsonl").write_text("\n".join(out) + "\n")

    def set_exit(root, tag, value):
        out = []
        for line in (root / "commands.jsonl").read_text().splitlines():
            e = json.loads(line)
            if e["tag"] == tag:
                e["exit"] = value
            out.append(json.dumps(e, sort_keys=True))
        (root / "commands.jsonl").write_text("\n".join(out) + "\n")

    injections = {
        "artifact_byte_flip": lambda r: flip_byte(r / "gen" / "C2.smc"),
        "missing_log": lambda r: (r / "logs" / "build.C1.verify.stdout").unlink(),
        "extra_file": lambda r: (r / "gen" / "C9.smc").write_bytes(b"x"),
        "verify_claim_flip": lambda r: mutate_json(r / "verify.json", lambda o: o["comparisons"][0].__setitem__("equal", not o["comparisons"][0]["equal"])),
        "tool_identity": lambda r: mutate_json(r / "preflight.json", lambda o: o["tools"]["smc"].__setitem__("sha256", "0" * 64)),
        "c0_dirty": lambda r: mutate_json(r / "preflight.json", lambda o: o["c0_checkout"].__setitem__("clean", False)),
        "verifier_rejects": lambda r: set_exit(r, "build.C2.verify", 1),
        "exit_zero_but_diagnostic": fake_diag,
        "row_removed": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].pop()),
        "row_duplicated": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"].append(o["rows"][0])),
        "expectation_rewritten": lambda r: mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("expect", "reject:type:x")),
        "snapshot_mutated": lambda r: flip_byte(r / "snapshot" / "lexer.sm"),
        "status_forged": lambda r: (mutate_json(r / "differential.json", lambda o: o["rows"][0].__setitem__("pass", False))),
        "vm_run_failed": lambda r: set_exit(r, next(e["tag"] for e in map(json.loads, (r / "commands.jsonl").read_text().splitlines()) if e["tag"].endswith(".run_smc")), 3),
    }
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
    print(f"checker self-test: {out['result']}")
    return 0 if ok else 1


def main(argv):
    if argv and argv[0] == "--selftest":
        return selftest(argv[1], argv[2])
    c = check(argv[0])
    if "--write" in argv:
        write_outputs(c)
    print(f"check_results: {'PASS' if not c.violations else 'FAIL'} ({len(c.violations)} violations)")
    for v in c.violations[:20]:
        print("  - " + v)
    return 0 if not c.violations else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
