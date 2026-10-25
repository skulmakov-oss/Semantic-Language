#!/usr/bin/env python3
"""Regression test suite for Bootstrap Contract v2 (PR #35).

Proves fail-closed rejection for:
1. v2 frozen digest mutation
2. Reference C0 SHA mismatch/tampering
3. Unverified capability claims (e.g. artifact_write available_at_c0)
4. Stale v1 evidence binding evaluated against v2 contract
5. Protocol immutability violations (v2 digest altered in place vs base)

Exit 0 on pass, non-zero on failure.
"""
import copy
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bootstrap"))

from validate_contract import (
    PROTOCOL,
    FROZEN_DIGESTS,
    CONTRACT,
    CONTRACT_DOC,
    check_contract,
    check_record,
    check_protocol_immutability,
)


def run_tests():
    contract = tomllib.loads((ROOT / CONTRACT).read_text(encoding="utf-8"))
    reference = tomllib.loads((ROOT / contract["c0"]["reference_manifest"]).read_text(encoding="utf-8"))
    subset_text = (ROOT / contract["subset"]["registry"]).read_text(encoding="utf-8")
    contract_doc = (ROOT / CONTRACT_DOC).read_text(encoding="utf-8")

    assert contract.get("protocol") == "shf0-bootstrap-contract-v2", "Expected active protocol v2"
    assert "shf0-bootstrap-contract-v1" in FROZEN_DIGESTS, "Expected historical v1 digest preserved"
    assert "shf0-bootstrap-contract-v2" in FROZEN_DIGESTS, "Expected v2 digest registered"

    # 1. Probe: v2 digest mutation
    errs1 = check_contract(contract, reference, subset_text, contract_doc,
                           registry={**FROZEN_DIGESTS, PROTOCOL: "0" * 64})
    assert any("a frozen value of shf0-bootstrap-contract-v2 changed" in e for e in errs1), errs1
    print("[test:PASS] Probe 1: v2 digest mutation rejected fail-closed")

    # 2. Probe: reference C0 substitution / SHA mismatch
    ref_tampered = copy.deepcopy(reference)
    ref_tampered["sha"] = "0" * 40
    errs2 = check_contract(contract, ref_tampered, subset_text, contract_doc)
    assert any("REFERENCE_MISMATCH: contract C0 SHA differs from the reference manifest" in e for e in errs2), errs2
    print("[test:PASS] Probe 2: reference C0 substitution rejected")

    # 3. Probe: unadmitted capability claim
    c_cap = copy.deepcopy(contract)
    c_cap["capabilities"]["artifact_write"]["available_at_c0"] = True
    errs3 = check_contract(c_cap, reference, subset_text, contract_doc)
    assert any("a frozen value of shf0-bootstrap-contract-v2 changed" in e for e in errs3), errs3
    print("[test:PASS] Probe 3: unadmitted capability claim rejected")

    # 4. Probe: stale v1 evidence binding against v2 contract
    h = "sha256:" + "ab" * 32
    c0_sha = contract["c0"]["sha"]
    rec4 = dict(
        contract_protocol="shf0-bootstrap-contract-v1",
        comparison_rule="byte-equality-v1",
        c0_identity=f"skulmakov-oss/Semantic@{c0_sha}",
        source_set_identity="sha256:" + "cd" * 32,
        c1_artifact_hash=h,
        c2_artifact_hash=h,
        c1_verifier_binding="admitted",
        c2_verifier_binding="admitted",
        comparison_result="equal",
        platform="x86_64-pc-windows-msvc",
        verifier_contract=contract["evidence"]["verifier_contract"],
        runtime_contract=contract["evidence"]["runtime_contract"],
        remaining_rust_responsibilities=["oracle", "host_mechanics", "verifier_runtime_foundation"],
        input_corpus="sha256:" + "12" * 32,
        positive_cases={"count": 30, "result": "pass"},
        negative_cases={"count": 8, "result": "pass"},
        boundary_cases={"count": 4, "result": "pass"},
        mutation_proof="detected",
        unexplained_deltas=0,
        input_corpus_size=42,
        limitations=["Windows x64 only (C0 platform scope)"],
        c0_checkout_head=c0_sha,
        c0_checkout_clean=True,
    )
    errs4 = check_record(contract, reference, rec4, "sha256:" + "cd" * 32)
    assert any("CONTRACT_DRIFT: evidence bound to a different contract protocol" in e for e in errs4), errs4
    print("[test:PASS] Probe 4: stale v1 evidence binding rejected against active v2 contract")

    # 5. Probe: protocol immutability catches v2 digest mutation vs base
    v1_h = FROZEN_DIGESTS["shf0-bootstrap-contract-v1"]
    v2_h = FROZEN_DIGESTS["shf0-bootstrap-contract-v2"]
    base_src = f'V1 = "shf0-bootstrap-contract-v1"\nV2 = "shf0-bootstrap-contract-v2"\nFROZEN_DIGESTS = {{V1: "{v1_h}", V2: "{v2_h}"}}\n'
    head_mutated = base_src.replace(v2_h, "f" * 64)
    errs5 = check_protocol_immutability(base_src, head_mutated)
    assert any("CONTRACT_DRIFT: frozen protocol shf0-bootstrap-contract-v2 was redefined in place" in e for e in errs5), errs5
    print("[test:PASS] Probe 5: protocol immutability catches v2 digest mutation vs base")

    print("\nSummary: 5/5 Bootstrap Contract v2 negative probes PASSED.")


if __name__ == "__main__":
    run_tests()
