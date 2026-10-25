"""Reference outcome comparison, separate from process orchestration."""


def outcome_key(outcome):
    if outcome["kind"] not in ("ok", "trap"):
        raise ValueError("Only admitted execution results can establish execution parity")
    if outcome["kind"] == "trap":
        return ("trap", outcome["detail"].splitlines()[-1] if outcome["detail"] else "")
    return (outcome["kind"], "")


def reference_matches(expected, outcome):
    detail = outcome.get("detail", "")
    if any(marker in detail.lower() for marker in (
            "internal compiler error", "panicked at", "traceback", "fatal runtime error")):
        return False
    if expected.startswith("reject:"):
        return (outcome.get("kind") == "compile_error" and outcome.get("compile_rc") == 1
                and outcome.get("artifact_exists") is False and expected[7:] in detail)
    if outcome.get("compile_rc") != 0 or outcome.get("verify_rc") != 0:
        return False
    if expected == "ok":
        return outcome.get("kind") == "ok" and outcome.get("run_rc") == 0
    if expected.startswith("trap:"):
        return (outcome.get("kind") == "trap" and outcome.get("run_rc") == 1
                and expected[5:] in detail)
    return False


def reference_pair_matches(expected, left, right):
    if not reference_matches(expected, left) or not reference_matches(expected, right):
        return False
    if expected.startswith("reject:"):
        return left["detail"] == right["detail"]
    return outcome_key(left) == outcome_key(right)
