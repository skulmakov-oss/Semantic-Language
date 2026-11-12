#!/usr/bin/env python3
"""Historical regression probes for Bootstrap Contract v2 (PR #35) under active v3.

v2 is a frozen, inactive protocol: only its registered digest survives in this repository.
v3 is the only active protocol. The five negative probes of the original v2 suite are kept,
each bound to the layer that enforces it today:

1. v2 digest mutation or removal    -> registry consistency and protocol immutability
2. Reference C0 SHA tampering       -> inherited reference-integrity invariant, evaluated
                                       under the active v3 contract (v2 is not executed)
3. artifact_write claimed available -> rejected by the active v3 frozen digest
4. Stale v1 / v2 evidence binding   -> rejected by active v3 admission
5. v2 digest redefined in place     -> specific CONTRACT_DRIFT from protocol immutability

The expected historical digests are pinned in this file, independently of the validator
registry they are compared with. This suite does NOT reconstruct the v2 digest: the archived
v2 normative inputs are not the current v3 documents, so no claim is made that the current
tree reproduces it. No validator state is modified; every mutation is applied to a copy.

Exit 0 on pass, non-zero on failure.
"""
import copy
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bootstrap"))

import validate_contract as vc  # noqa: E402

V1, V2, V3 = (f"shf0-bootstrap-contract-v{n}" for n in (1, 2, 3))
# Independent pins: never read back from the registry under test.
PINNED = {
    V1: "6f80db639198bfcb90ec49678024e8a793ab497b1300bc308eb2063543027908",
    V2: "8a2b910989c55e871c6099c673c6a1c65fbaffb5f8ac5eb876466fe9b7fd727b",
    V3: "26309eb87a8e765407d771f174e0f37966d600b178f4c0d164cfbe351e3925f6",
}
OTHER_DIGEST = "f" * 64  # a well-formed digest that is not the v2 one
DRIFT_PROTOCOL = "CONTRACT_DRIFT: evidence bound to a different contract protocol"
DRIFT_REGISTRY = "CONTRACT_DRIFT: FROZEN_DIGESTS is modified after its literal definition"


def run_tests():
    contract = tomllib.loads((ROOT / vc.CONTRACT).read_text(encoding="utf-8"))
    reference = tomllib.loads((ROOT / contract["c0"]["reference_manifest"]).read_text(encoding="utf-8"))
    subset_text = (ROOT / contract["subset"]["registry"]).read_text(encoding="utf-8")
    contract_doc = (ROOT / vc.CONTRACT_DOC).read_text(encoding="utf-8")
    validator_source = Path(vc.__file__).read_text(encoding="utf-8")

    def check(c=contract, r=reference, **kw):
        return vc.check_contract(c, r, subset_text, contract_doc, **kw)

    # Historical protocol identity: v3 active, v2 registered and inactive, all digests pinned,
    # literal registry == parsed source == runtime registry.
    assert vc.PROTOCOL == V3 and contract.get("protocol") == V3, "Expected active protocol v3"
    assert V2 != vc.PROTOCOL and V2 in vc.FROZEN_DIGESTS, "Expected v2 registered as historical"
    assert vc.FROZEN_DIGESTS == PINNED, f"frozen registry differs from the pinned digests: {vc.FROZEN_DIGESTS}"
    assert vc.literal_frozen_digests() == PINNED, "literal registry differs from the pinned digests"
    assert vc.read_frozen_digests(validator_source) == PINNED, "parsed registry differs from the pinned digests"
    assert validator_source.count(PINNED[V2]) == 1, "v2 digest must be written exactly once in the validator"
    # Clean baseline: every rejection below is caused by its own mutation.
    assert check() == [], check()
    assert vc.check_protocol_immutability(validator_source, validator_source) == []
    print("[test:PASS] Identity: v3 active; v2 historical; v1/v2/v3 digests pinned; literal == parsed == runtime")

    # 1. Probe: historical v2 digest mutated or removed
    altered = {**PINNED, V2: OTHER_DIGEST}
    removed = {k: v for k, v in PINNED.items() if k != V2}
    for registry in (altered, removed):  # literal registry disagrees with the runtime registry
        assert DRIFT_REGISTRY in check(registry=registry), check(registry=registry)
    src_removed = "\n".join(line for line in validator_source.splitlines() if PINNED[V2] not in line)
    assert V2 not in vc.read_frozen_digests(src_removed), "mutation did not remove the v2 entry"
    errs1 = vc.check_protocol_immutability(validator_source, src_removed)
    assert errs1 == [f"CONTRACT_DRIFT: frozen protocol {V2} was removed"], errs1
    print("[test:PASS] Probe 1: historical v2 digest mutation and removal rejected fail-closed")

    # 2. Probe: reference C0 substitution / SHA mismatch (inherited invariant, active v3 contract)
    ref_tampered = copy.deepcopy(reference)
    ref_tampered["sha"] = "0" * 40
    errs2 = check(r=ref_tampered)
    assert "REFERENCE_MISMATCH: contract C0 SHA differs from the reference manifest" in errs2, errs2
    print("[test:PASS] Probe 2: reference C0 substitution rejected (inherited invariant under active v3)")

    # 3. Probe: unadmitted capability claim (artifact_write is a declared gap at C0)
    assert contract["capabilities"]["artifact_write"]["available_at_c0"] is False
    c_cap = copy.deepcopy(contract)
    c_cap["capabilities"]["artifact_write"]["available_at_c0"] = True
    errs3 = check(c=c_cap)
    assert any(e.startswith(f"CONTRACT_DRIFT: a frozen value of {V3} changed") for e in errs3), errs3
    print("[test:PASS] Probe 3: unadmitted artifact_write availability rejected by the active contract")

    # 4. Probe: evidence bound to an obsolete protocol, evaluated by active v3 admission.
    # Historical v2 admission is not executed here; only "v1/v2 evidence is stale under v3".
    h = "sha256:" + "ab" * 32
    identity = "sha256:" + "cd" * 32
    c0_sha = contract["c0"]["sha"]
    record = dict(
        contract_protocol=V3,
        comparison_rule="byte-equality-v1",
        c0_identity=f"skulmakov-oss/Semantic@{c0_sha}",
        source_set_identity=identity,
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
    control = vc.check_record(contract, reference, record, identity)
    assert control == [], control  # the same record bound to v3 is well-formed
    for stale in (V1, V2):
        errs4 = vc.check_record(contract, reference, {**record, "contract_protocol": stale}, identity)
        assert errs4 == [DRIFT_PROTOCOL], (stale, errs4)  # the binding is the only defect
    print("[test:PASS] Probe 4: stale v1 / v2 evidence binding rejected against active v3 contract")

    # 5. Probe: protocol immutability catches the v2 digest redefined in place vs base
    head_mutated = validator_source.replace(PINNED[V2], OTHER_DIGEST)
    assert vc.read_frozen_digests(head_mutated) == altered, "mutation did not redefine the v2 entry"
    errs5 = vc.check_protocol_immutability(validator_source, head_mutated)
    assert errs5 == [f"CONTRACT_DRIFT: frozen protocol {V2} was redefined in place "
                     "(introduce a new protocol identifier instead)"], errs5
    print("[test:PASS] Probe 5: protocol immutability catches v2 digest redefinition vs base")

    print("\nSummary: 5/5 Bootstrap Contract v2 historical probes PASSED under active v3.")


if __name__ == "__main__":
    run_tests()
