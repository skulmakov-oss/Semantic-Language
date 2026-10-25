"""End-to-end u16 name boundaries; generated inputs are not canonical bootstrap S."""
import json
import os
import pathlib
import subprocess
import sys

from reference_identity import qualify_reference


def main():
    repo = pathlib.Path(__file__).resolve().parents[2]
    smc = pathlib.Path(os.environ["SEMANTIC_C0_SMC"])
    identity = qualify_reference(repo, smc, smc, os.environ)
    out = pathlib.Path(os.environ["SEMANTIC_TEST_OUT"]).resolve()
    if out == repo or repo in out.parents:
        raise ValueError("boundary evidence must be outside the repository")
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    cases = [
        ("name_65536", "fn " + "a" * 65536 + "() { return; }\nfn main() { return; }\n", False, "main.sm"),
        ("name_65535", "fn " + "a" * 65535 + "() { return; }\nfn main() { return; }\n", False, "main.sm"),
        ("name_1025", "fn " + "a" * 1025 + "() { return; }\nfn main() { return; }\n", False, "main.sm"),
        ("name_1024", "fn " + "a" * 1024 + "() { return; }\nfn main() { return; }\n", True, "main.sm"),
        ("local_suffix_1025", "fn main() { let " + "a" * 1023 + ": i32 = 1; return; }\n", False, "main.sm"),
        ("local_suffix_1024", "fn main() { let " + "a" * 1022 + ": i32 = 1; return; }\n", True, "main.sm"),
        ("utf8_string_65536", "é" * 32768, False, "../tests/compiler/programs/str16_encoding.sm"),
        ("utf8_string_1025", "é" * 512 + "a", False, "../tests/compiler/programs/str16_encoding.sm"),
        ("utf8_string_1024", "é" * 512, True, "../tests/compiler/programs/str16_encoding.sm"),
        ("table_256", "v", True, "../tests/compiler/programs/str16_encoding.sm"),
        ("table_257", "v", False, "../tests/compiler/programs/str16_encoding.sm"),
    ]
    for name, source, accepted, entry in cases:
        case = out / (name + ".sm")
        case.write_text(source, encoding="utf-8", newline="\n")
        args = [str(smc), "run", entry, "--profile", "cli-read-only", "--root", str(out),
                "--envelope", "trusted-compiler", "--max-steps", "20000000", "--max-calls", "50000",
                "--max-frames", "256", "--metrics", "json", "--", case.name,
                "256" if name == "table_256" else "257" if name == "table_257" else "1"]
        result = subprocess.run(args, cwd=repo / "compiler", capture_output=True, text=True,
                                encoding="utf-8", timeout=180)
        (out / (name + ".stdout")).write_text(result.stdout, encoding="utf-8")
        (out / (name + ".stderr")).write_text(result.stderr, encoding="utf-8")
        repeated = subprocess.run(args, cwd=repo / "compiler", capture_output=True, text=True,
                                  encoding="utf-8", timeout=180)
        correct = result.returncode == 0 and (result.stdout.startswith("hex ") if accepted
                                             else result.stdout.startswith("error unsupported @")
                                             and ("256 entries" if name == "table_257" else "UTF-8 bytes") in result.stdout
                                             and not result.stdout.startswith("hex "))
        correct = correct and (repeated.returncode, repeated.stdout) == (result.returncode, result.stdout)
        admission = []
        if correct and accepted:
            artifact = out / (name + ".smc")
            artifact.write_bytes(bytes.fromhex(result.stdout[4:].strip()))
            for stage in ("verify", "run-smc"):
                checked = subprocess.run([str(smc), stage, str(artifact)], capture_output=True,
                                         text=True, encoding="utf-8", timeout=180)
                admission.append(dict(stage=stage,exit_code=checked.returncode,stdout=checked.stdout,stderr=checked.stderr))
                if checked.returncode:
                    correct = False
                    break
        rows.append(dict(case=name,source_bytes=len(source.encode("utf-8")),command=args,
                         exit_code=result.returncode,repeated_exit_code=repeated.returncode,
                         admission=admission,passed=correct))
        (out / "results.json").write_text(json.dumps(dict(identity=identity,rows=rows),indent=2),encoding="utf-8")
        print(f"{'PASS' if correct else 'FAIL'} {name}, exit={result.returncode}",flush=True)
        if not correct:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
