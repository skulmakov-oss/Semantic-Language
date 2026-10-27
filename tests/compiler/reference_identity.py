"""Producer-side C0 checks. Unsigned build attestations are not a Bootstrap Seal."""
import hashlib
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import time
import tomllib


class _IdentityDirectory(tempfile.TemporaryDirectory):
    def cleanup(self):
        # Windows can retain a sharing lock briefly after the helper exits.
        for attempt in range(101):
            try:
                return super().cleanup()
            except PermissionError as error:
                if getattr(error, "winerror", None) != 32 or attempt == 100:
                    raise
                time.sleep(0.1)


def file_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def command(args, **kwargs):
    result = subprocess.run([str(a) for a in args], capture_output=True, text=True,
                            encoding="utf-8", errors="strict", timeout=120, **kwargs)
    if result.returncode:
        raise ValueError(f"command failed ({result.returncode}): {args[0]}: {result.stderr}")
    return result.stdout.strip()


def qualify_reference(repo, smc, c0, env):
    """Validate source, contract, build attestation and both supplied executables before use."""
    try:
        reference_path = repo / "reference/semantic-reference.toml"
        contract_path = repo / "bootstrap/contract.toml"
        reference = tomllib.loads(reference_path.read_text(encoding="utf-8"))
        contract = tomllib.loads(contract_path.read_text(encoding="utf-8"))
        if (reference["repository"] != "skulmakov-oss/Semantic"
                or reference["status"] != "qualified-reference"
                or contract["protocol"] != "shf0-bootstrap-contract-v2"
                or contract["status"] != "frozen"
                or contract["c0"]["sha"] != reference["sha"]):
            raise ValueError("reference / frozen Bootstrap Contract v2 identity mismatch")
        paths = {}
        for key in ("SEMANTIC_C0_CHECKOUT", "SEMANTIC_C0_PROVENANCE", "SEMANTIC_C0_ANCHOR"):
            if not env.get(key):
                raise ValueError(f"{key} is required; a binary path/hash/version alone is insufficient")
            paths[key] = pathlib.Path(env[key]).resolve(strict=True)
        checkout = paths["SEMANTIC_C0_CHECKOUT"]
        spec = importlib.util.spec_from_file_location("bootstrap_reference_check", repo / "bootstrap/validate_contract.py")
        validator = importlib.util.module_from_spec(spec)
        old_bytecode = sys.dont_write_bytecode
        try:
            sys.dont_write_bytecode = True
            spec.loader.exec_module(validator)
        finally:
            sys.dont_write_bytecode = old_bytecode
        errors = validator.check_contract(
            contract, reference,
            (repo / "docs/BOOTSTRAP_SUBSET.md").read_text(encoding="utf-8"),
            (repo / "docs/BOOTSTRAP_CONTRACT.md").read_text(encoding="utf-8"))
        if errors:
            raise ValueError("; ".join(errors))
        errors = validator.verify_reference_checkout(checkout, reference["sha"])
        if errors:
            raise ValueError("; ".join(errors))
        remote = command(["git", "-C", checkout, "remote", "get-url", "origin"])
        if remote.removesuffix(".git").rstrip("/") not in (
                "https://github.com/skulmakov-oss/Semantic", "git@github.com:skulmakov-oss/Semantic"):
            raise ValueError("reference checkout origin is not the canonical repository")
        provenance = json.loads(paths["SEMANTIC_C0_PROVENANCE"].read_text(encoding="utf-8"))
        host_provenance_path = pathlib.Path(env.get("SEMANTIC_SMC_PROVENANCE", paths["SEMANTIC_C0_PROVENANCE"]))
        host_provenance = json.loads(host_provenance_path.read_text(encoding="utf-8"))
        anchor = json.loads(paths["SEMANTIC_C0_ANCHOR"].read_text(encoding="utf-8"))
        # Distribution metadata names the Git blob, not CRLF-expanded checkout bytes.
        lock_blob = subprocess.run(["git", "-C", str(checkout), "show", "HEAD:Cargo.lock"],
                                   capture_output=True, check=True).stdout
        lock_hash = hashlib.sha256(lock_blob).hexdigest()
        lock_oid = command(["git", "-C", checkout, "rev-parse", "HEAD:Cargo.lock"])
        for binding in (provenance["upstream"], host_provenance["upstream"], anchor["canonical_upstream"]):
            if (binding["repository_url"] != "https://github.com/skulmakov-oss/Semantic"
                    or binding["pinned_commit_sha"] != reference["sha"]
                    or binding["cargo_lock_blob_sha256"] != lock_hash
                    or binding["cargo_lock_blob_git_oid"] != lock_oid):
                raise ValueError("build/release provenance does not bind the exact reference and lockfile")
        if any(p["build_environment"]["target_triple"] != "x86_64-pc-windows-msvc"
               for p in (provenance, host_provenance)) or os.name != "nt":
            raise ValueError("this Windows qualification attestation cannot qualify experimental WSL execution")
        recorded = provenance["artifacts"]["binaries"]["smc.exe"]
        for binary, build in ((smc, host_provenance), (c0, provenance)):
            artifact = build["artifacts"]["binaries"]["smc.exe"]
            if file_hash(binary) != artifact["sha256"] or binary.stat().st_size != artifact["bytes"]:
                raise ValueError("supplied compiler binary differs from the attested build artifact")
        # Execute the pinned build-identity implementation, not a host reimplementation of its algorithm.
        with _IdentityDirectory(prefix="semantic-c0-identity-") as directory:
            identity = pathlib.Path(directory) / "identity.exe"
            command(["rustc", "--edition=2021", checkout / "crates/smc-cli/build.rs", "-o", identity])
            identity_env = os.environ.copy()
            identity_env["CARGO_MANIFEST_DIR"] = str(checkout / "crates/smc-cli")
            fingerprint_output = command([identity], env=identity_env)
        fingerprint = next(line.split("=", 2)[2] for line in fingerprint_output.splitlines()
                           if line.startswith("cargo:rustc-env=SM_COMPILER_SOURCE_HASH="))
        versions = [json.loads(command([binary, "version", "--json"])) for binary in (smc, c0)]
        if any(v.get("source_fingerprint") != fingerprint for v in versions):
            raise ValueError("binary source fingerprint does not match the pinned source computation")
        if any(not {"STD", "PROFILE_RUST"}.issubset(v.get("enabled_features", [])) for v in versions):
            raise ValueError("binary lacks required source/execution features")
        errors = validator.verify_reference_checkout(checkout, reference["sha"])
        if errors:
            raise ValueError("reference drift during identity verification: " + "; ".join(errors))
        return dict(repository=reference["repository"], sha=reference["sha"], checkout=str(checkout),
                    checkout_clean=True, source_fingerprint=fingerprint,
                    binary_sha256=recorded["sha256"], version=versions[1],
                    host_binary_sha256=file_hash(smc),
                    contract_protocol=contract["protocol"], contract_sha256=file_hash(contract_path),
                    reference_manifest_sha256=file_hash(reference_path),
                    cargo_lock_blob_sha256=lock_hash,
                    cargo_lock_checkout_sha256=file_hash(checkout / "Cargo.lock"),
                    provenance_sha256=file_hash(paths["SEMANTIC_C0_PROVENANCE"]),
                    host_provenance_sha256=file_hash(host_provenance_path),
                    anchor_sha256=file_hash(paths["SEMANTIC_C0_ANCHOR"]),
                    platform="x86_64-pc-windows-msvc",
                    provenance_status="unsigned-source-build-attestation" if provenance["workflow_run_identity"]["github_run_id"] == "local" else "unsigned-distribution-build-attestation",
                    limitations=["Producer-side checks and unsigned metadata are not cryptographic execution proof or independent Seal reproduction.",
                                 "When both binary hashes match, the two runner roles are not independent implementations."])
    except (ValueError, OSError, KeyError, TypeError, AttributeError, StopIteration, subprocess.SubprocessError) as error:
        raise ValueError(f"REFERENCE_MISMATCH: {error}") from error
