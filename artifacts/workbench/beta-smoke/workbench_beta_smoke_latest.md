# Workbench Beta Smoke Report

- Generated: 2026-03-15T21:22:42.0734627Z
- Branch: codex/wb-bh-14-beta-release-notes
- Commit: 2ea476e3f8c554daa958fccaf88096b1c4583bd8
- Output root: artifacts/workbench/beta-smoke
- Workspace root: artifacts/workbench/beta-smoke/workspace
- Package format: portable-zip
- Portable zip: artifacts/workbench/beta-smoke/package/semantic-workbench-beta-portable.zip
- Release executable: artifacts/workbench/beta-smoke/tauri-target/release/semantic-workbench-app.exe
- Launch smoke: passed (8 s window)

## Acceptance Coverage

- Packaged app launched from the extracted portable beta package.
- Smoke loop covered diagnostics, format check/write, compile, verify, disasm, run, and release bundle verification.
- Full command captures are preserved under 'artifacts/workbench/beta-smoke/logs/'.

## Step Summary

| Step | Expectation | Exit | Duration (ms) | Stdout | Stderr |
| --- | --- | ---: | ---: | --- | --- |
| workbench lint | success | 0 | 6048 | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stderr.txt' |
| workbench build | success | 0 | 6024 | 'artifacts/workbench/beta-smoke/logs/workbench-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-build.stderr.txt' |
| workbench tauri tests | success | 0 | 2035 | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stderr.txt' |
| semantic release binaries | success | 0 | 1022 | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stderr.txt' |
| workbench release build | success | 0 | 358029 | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stderr.txt' |
| smoke diagnostics check | expected failure | 1 | 1020 | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stderr.txt' |
| smoke format check before write | expected failure | 1 | 1023 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stderr.txt' |
| smoke format write | success | 0 | 1022 | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stderr.txt' |
| smoke format check after write | success | 0 | 1019 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stderr.txt' |
| smoke compile | success | 0 | 1024 | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stderr.txt' |
| smoke verify | success | 0 | 1017 | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stderr.txt' |
| smoke disasm | success | 0 | 1018 | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stderr.txt' |
| smoke run | success | 0 | 1015 | 'artifacts/workbench/beta-smoke/logs/smoke-run.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-run.stderr.txt' |
| release bundle verify | success | 0 | 1013 | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stderr.txt' |

## Bundle Inventory

| Artifact | Size (bytes) | SHA256 |
| --- | ---: | --- |
| artifacts/workbench/beta-smoke/tauri-target/release/semantic-workbench-app.exe | 9077248 | 'ce6076ac0a173301c63960c05410a1036799317e5d609966987a3aad41d9c2a8' |
| artifacts/workbench/beta-smoke/package/semantic-workbench-beta-portable.zip | 2895803 | '9850e18d3a83037e272642796b2333aa6dd44d81ddc1bf71abaa3d5964dd6820' |
