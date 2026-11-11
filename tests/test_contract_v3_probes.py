#!/usr/bin/env python3
"""Probe suite for Bootstrap Contract v3 (SHF-0R P).

Each probe asserts one property of the v3 contract. "Substantive" probes ignore the generic
frozen-digest error, so a pass or failure names the actual authority / capability defect and
not merely "some frozen value changed".

D1  exact upstream authorities for BSF-101 / BSF-103, documented fields == contract.toml
D2  BSF-103 carries no bitwise / shift / conversion claim; BSF-107 stays CANDIDATE
D3  VerifiedLocal default quotas are distinguished from the declared trusted-compiler envelope
P   v3 is the active protocol; v1 / v2 digests are preserved; security invariants hold

Exit 0 when every probe passes, 1 otherwise.
"""
import copy
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bootstrap"))

import validate_contract as vc  # noqa: E402

V1, V2, V3 = (f"shf0-bootstrap-contract-v{n}" for n in (1, 2, 3))
V1_DIGEST = "6f80db639198bfcb90ec49678024e8a793ab497b1300bc308eb2063543027908"
V2_DIGEST = "8a2b910989c55e871c6099c673c6a1c65fbaffb5f8ac5eb876466fe9b7fd727b"
C0 = "1d1e76c7509a03baa0133abaaa5819f5d32582b2"
REPO = "skulmakov-oss/Semantic"
AUTHORITIES = {
    "BSF-101": {
        "repository": REPO, "sha": C0,
        "path": "docs/spec/compiler_text_v0.md",
        "contract_id": "semantic.compiler.text/0.1",
        "qualification": ["tests/shf1b_compiler_text_qualification.rs", "tests/shf1c_lexer_probe.rs"],
    },
    "BSF-103": {
        "repository": REPO, "sha": C0,
        "path": "docs/spec/compiler_u32_v0.md",
        "contract_id": "semantic.compiler.u32/0.1",
        "qualification": ["tests/shf3a_u32_qualification.rs"],
    },
}
TEXT_OPS = ["text_len", "text_byte_at", "text_slice", "text_starts_with", "text_ends_with",
            "text_find", "text_is_empty"]
ENVELOPE = "--envelope trusted-compiler --max-calls 50000 --max-steps 20000000"

contract = tomllib.loads((ROOT / vc.CONTRACT).read_text(encoding="utf-8"))
reference = tomllib.loads((ROOT / contract["c0"]["reference_manifest"]).read_text(encoding="utf-8"))
subset = (ROOT / contract["subset"]["registry"]).read_text(encoding="utf-8")
doc = (ROOT / vc.CONTRACT_DOC).read_text(encoding="utf-8")
validator_source = (ROOT / "bootstrap/validate_contract.py").read_text(encoding="utf-8")


def errors(c=None, r=None, s=None, d=None):
    return vc.check_contract(contract if c is None else c, reference if r is None else r,
                             subset if s is None else s, doc if d is None else d)


def substantive(**kw):
    """check_contract errors without the generic frozen-digest one."""
    return [e for e in errors(**kw) if "frozen value" not in e]


def with_contract(fn):
    c = copy.deepcopy(contract)
    fn(c)
    return c


def row(bsf):
    found = [cells for i, _, cells in vc.registry_rows(subset) if i == bsf]
    assert len(found) == 1, f"{bsf}: expected exactly one registry row, found {len(found)}"
    return found[0]


def section_10_5():
    start = doc.index("### 10.5")
    return doc[start:doc.index("### 10.6", start)]


def authority_rejected(errs, why):
    assert any("subset authority" in e for e in errs), f"{why}: not rejected as an authority defect: {errs}"


# ---------------------------------------------------------------- D1
def d1_bsf101_exact_authority():
    assert contract["subset"].get("authorities", {}).get("BSF-101") == AUTHORITIES["BSF-101"]
    assert all(f"`{op}`" in row("BSF-101")[1] for op in TEXT_OPS), "BSF-101 must name its seven operations"


def d1_bsf103_exact_authority():
    assert contract["subset"].get("authorities", {}).get("BSF-103") == AUTHORITIES["BSF-103"]


def d1_only_additive_entries_have_authorities():
    assert set(contract["subset"].get("authorities", {})) == set(AUTHORITIES)


def d1_documented_authorities_equal_toml():
    assert substantive() == [], substantive()
    for field in ("contract_id", "path"):
        value = AUTHORITIES["BSF-103"][field]
        assert value in subset, f"registry does not document {value}"
        authority_rejected(substantive(s=subset.replace(value, value + "x")), f"documented {field} altered")
    assert C0 in subset
    for bsf in AUTHORITIES:  # a documented authority line may not simply disappear
        kept = "\n".join(l for l in subset.splitlines() if not (l.startswith("- ") and bsf in l))
        authority_rejected(substantive(s=kept), f"documented authority of {bsf} removed")


def d1_missing_or_altered_authority_fields_rejected():
    for field in AUTHORITIES["BSF-101"]:
        c = with_contract(lambda c, f=field: c["subset"]["authorities"]["BSF-101"].pop(f))
        authority_rejected(substantive(c=c), f"missing {field}")
    for field, bad in (("repository", "someone/Semantic"), ("sha", "0" * 40), ("sha", "main"),
                       ("path", "docs/spec/other.md"), ("contract_id", "semantic.compiler.u32/9.9"),
                       ("contract_id", ""), ("qualification", []),
                       ("qualification", ["tests/other.rs"]), ("qualification", "tests/x.rs")):
        c = with_contract(lambda c, f=field, b=bad: c["subset"]["authorities"]["BSF-103"].update({f: b}))
        authority_rejected(substantive(c=c), f"{field}={bad!r}")
    authority_rejected(substantive(c=with_contract(lambda c: c["subset"]["authorities"].pop("BSF-103"))),
                       "BSF-103 authority missing")
    authority_rejected(substantive(c=with_contract(lambda c: c["subset"].pop("authorities"))),
                       "authorities table missing")
    for unknown in ("BSF-102", "BSF-107", "BSF-999", "BSF-001"):  # candidate / unlisted / foundation
        c = with_contract(lambda c, u=unknown: c["subset"]["authorities"].update(
            {u: dict(AUTHORITIES["BSF-103"])}))
        authority_rejected(substantive(c=c), f"authority for {unknown}")


def d1_coordinated_authority_substitution_rejected():
    """The same wrong authority on both sides (registry and TOML) is still rejected: the exact
    authority of each additive entry is pinned, not merely kept consistent."""
    for field, good, bad in (("contract_id", "semantic.compiler.u32/0.1", "semantic.compiler.u32/0.2"),
                             ("path", "docs/spec/compiler_u32_v0.md", "docs/spec/nonexistent.md"),
                             ("qualification", "tests/shf3a_u32_qualification.rs", "tests/shf9z_other.rs")):
        c = with_contract(lambda c, f=field, b=bad: c["subset"]["authorities"]["BSF-103"].update(
            {f: [b] if f == "qualification" else b}))
        s = subset.replace(f"`{good}`", f"`{bad}`")
        assert s != subset
        authority_rejected(substantive(c=c, s=s), f"coordinated {field} -> {bad}")

    def swap(c):
        a = c["subset"]["authorities"]
        a["BSF-101"], a["BSF-103"] = a["BSF-103"], a["BSF-101"]
    s = subset.replace("- `BSF-101`", "- `BSF-x`").replace("- `BSF-103`", "- `BSF-101`").replace("- `BSF-x`", "- `BSF-103`")
    authority_rejected(substantive(c=with_contract(swap), s=s), "coordinated swap of authorities")


# ---------------------------------------------------------------- D2
def d2_bsf103_excludes_bitwise_shifts_conversions():
    cells = row("BSF-103")
    text = " ".join(cells).lower()
    for claim in ("bitwise", "bitmask", "shift", "conver", "coerci"):
        assert claim not in text, f"BSF-103 still claims {claim!r}: {cells}"
    assert cells[2] == "ADMITTED"
    assert cells[1] == "`u32` checked arithmetic `+ - * / %` and ordering `< <= > >=`", cells[1]
    assert "==" not in text and "!=" not in text and "equal" not in text, "equality is not introduced by BSF-103"
    assert "overflow" in text and "division" in text, "trap evidence must be preserved"


def d2_bsf107_is_the_single_candidate():
    cells = row("BSF-107")
    assert cells[2] == "CANDIDATE", cells
    assert cells[1] == "Deterministic u32 bitwise operations, shifts and related integer conversions", cells[1]
    assert REPO in cells[4] and "SHF-3" in " ".join(cells), cells
    sub = contract["subset"]
    assert "BSF-107" in sub["candidate"]
    assert "BSF-107" not in sub["admitted"] + sub["frozen"]
    assert sorted(sub["candidate"]) == ["BSF-102", "BSF-104", "BSF-105", "BSF-106", "BSF-107"]


def d2_implicit_promotion_rejected():
    def promote(c):
        c["subset"]["candidate"].remove("BSF-107")
        c["subset"]["admitted"].append("BSF-107")
    errs = substantive(c=with_contract(promote))                      # TOML only
    assert any("registry and contract disagree" in e for e in errs), errs
    line = next(l for l in subset.splitlines() if l.startswith("| BSF-107 |"))
    promoted = subset.replace(line, line.replace("| CANDIDATE |", "| ADMITTED |"))
    assert substantive(s=promoted), "registry-only promotion of BSF-107 accepted"
    both = subset.replace(line, "| BSF-107 | u32 bitwise | ADMITTED | use | T A F I B / traps | SHF-10..14 |")
    authority_rejected(substantive(c=with_contract(promote), s=both),   # consistent on both sides,
                       "BSF-107 admitted without an exact authority")   # but no authority


# ---------------------------------------------------------------- D3
def d3_verified_local_defaults():
    sec = section_10_5()
    for needle in ("VerifiedLocal", "100,000", "16,384", "256", f"{REPO}@{C0}", "docs/spec/quotas.md"):
        assert needle in sec, f"§10.5 lacks {needle!r}"
    assert "10,000,000" not in sec, "§10.5 still states 10,000,000 default steps"


def d3_trusted_compiler_envelope_is_explicit():
    sec = section_10_5()
    assert ENVELOPE in sec
    assert "not defaults" in sec and "explicit" in sec, "envelope must be marked an explicit selection"
    default_line = next(l for l in sec.splitlines() if "VerifiedLocal" in l and "100,000" in l)
    assert "50000" not in default_line and "20000000" not in default_line


# ---------------------------------------------------------------- protocol
def p_v3_is_active_with_exact_digest():
    assert vc.PROTOCOL == V3 and contract["protocol"] == V3
    assert vc.FROZEN_DIGESTS.get(V3) == vc.frozen_digest(contract, reference, subset, doc)
    assert vc.read_frozen_digests(validator_source) == vc.FROZEN_DIGESTS
    assert errors() == [], errors()


def p_v3_normative_mutation_rejected():
    changed = f"a frozen value of {V3} changed"
    for kw in (dict(s=subset.replace("ADMITTED", "ADMITTED ", 1).replace("## 3. Minimality", "## 3. Maximality")),
               dict(d=doc.replace("byte-equality-v1", "normalized-equality-v1", 1)),
               dict(d=doc.replace("100,000", "999,999", 1)),
               dict(c=with_contract(lambda c: c["compiler"].update(artifact_format="other")))):
        assert any(changed in e for e in errors(**kw)), kw.keys()


def p_v1_v2_digests_preserved():
    assert vc.FROZEN_DIGESTS.get(V1) == V1_DIGEST and vc.FROZEN_DIGESTS.get(V2) == V2_DIGEST
    for protocol, digest in ((V1, V1_DIGEST), (V2, V2_DIGEST)):
        assert validator_source.count(digest) == 1
        rewritten = validator_source.replace(digest, "0" * 64)
        errs = vc.check_protocol_immutability(validator_source, rewritten)
        assert any(f"{protocol} was redefined in place" in e for e in errs), errs
        deleted = "\n".join(l for l in validator_source.splitlines() if digest not in l)
        errs = vc.check_protocol_immutability(validator_source, deleted)
        assert any(f"{protocol} was removed" in e for e in errs), errs


def p_c0_identity_substitution_rejected():
    assert contract["c0"]["sha"] == C0 == reference["sha"]
    r = copy.deepcopy(reference)
    r["sha"] = "0" * 40
    assert any(e.startswith("REFERENCE_MISMATCH:") for e in substantive(r=r))
    c = with_contract(lambda c: c["c0"].update(sha="0" * 40))
    assert any(e.startswith("REFERENCE_MISMATCH:") for e in substantive(c=c))


def p_runtime_registry_mutation_rejected():
    saved = dict(vc.FROZEN_DIGESTS)
    try:
        vc.FROZEN_DIGESTS[vc.PROTOCOL] = "0" * 64
        errs = errors()
    finally:
        vc.FROZEN_DIGESTS.clear()
        vc.FROZEN_DIGESTS.update(saved)
    assert any("modified after its literal definition" in e for e in errs), errs


def p_source_set_mutation_rejected():
    manifest = tomllib.loads((ROOT / contract["source_set"]["manifest"]).read_text(encoding="utf-8"))
    root = contract["source_set"]["root"]
    identity, errs = vc.load_source_set(ROOT, manifest, root)
    assert identity and not errs, errs
    for files in (manifest["files"] + ["compiler/zzzz_not_declared_anywhere.sm"],
                  manifest["files"][::-1] if len(manifest["files"]) > 1 else ["compiler/b.sm", "compiler/a.sm"],
                  manifest["files"] + ["../outside.sm"]):
        got, errs = vc.load_source_set(ROOT, {**manifest, "files": files}, root)
        assert got is None and any(e.startswith("SOURCE_SET_INVALID:") for e in errs), files[-1]
    c = with_contract(lambda c: c["source_set"].update(newline="normalize"))
    assert any(e.startswith("SOURCE_SET_INVALID:") for e in substantive(c=c))


def p_host_capability_escalation_rejected():
    assert contract["capabilities"]["artifact_write"]["available_at_c0"] is False
    c = with_contract(lambda c: c["host"]["required_capabilities"].append("environment_read"))
    assert any(e.startswith("HOST_LOGIC_LEAK:") for e in substantive(c=c))
    c = with_contract(lambda c: c["host"]["forbidden_logic"].remove("parsing"))
    assert any(e.startswith("HOST_LOGIC_LEAK:") for e in substantive(c=c))
    c = with_contract(lambda c: c["capabilities"]["artifact_write"].pop("gap_stage"))
    assert any(e.startswith("CAPABILITY_GAP:") for e in substantive(c=c))
    # claiming the capability gap closed, or admitting a forbidden input, is bound by the v3 digest
    for fn in (lambda c: c["capabilities"]["artifact_write"].update(available_at_c0=True),
               lambda c: c["capabilities"]["environment_read"].update(permitted=True, available_at_c0=True)):
        assert any(f"a frozen value of {V3} changed" in e for e in errors(c=with_contract(fn)))


PROBES = [
    d1_bsf101_exact_authority, d1_bsf103_exact_authority, d1_only_additive_entries_have_authorities,
    d1_documented_authorities_equal_toml, d1_missing_or_altered_authority_fields_rejected,
    d1_coordinated_authority_substitution_rejected,
    d2_bsf103_excludes_bitwise_shifts_conversions, d2_bsf107_is_the_single_candidate,
    d2_implicit_promotion_rejected,
    d3_verified_local_defaults, d3_trusted_compiler_envelope_is_explicit,
    p_v3_is_active_with_exact_digest, p_v3_normative_mutation_rejected, p_v1_v2_digests_preserved,
    p_c0_identity_substitution_rejected, p_runtime_registry_mutation_rejected,
    p_source_set_mutation_rejected, p_host_capability_escalation_rejected,
]


def main():
    failed = 0
    for probe in PROBES:
        try:
            probe()
            print(f"[test:PASS] {probe.__name__}")
        except Exception as exc:  # a probe that cannot even run is a failure, not a skip
            failed += 1
            print(f"[test:FAIL] {probe.__name__}: {type(exc).__name__}: {str(exc)[:300]}")
    print(f"\nSummary: {len(PROBES) - failed}/{len(PROBES)} Bootstrap Contract v3 probes PASSED.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
