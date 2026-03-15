# Workbench Beta Smoke Report

- Generated: 2026-03-15T21:30:01.4975699Z
- Branch: codex/wb-bh-14-beta-release-notes
- Commit: a39a01f30e03f1031dfb9cf095987b5ed1bad5ea
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
| workbench lint | success | 0 | 7062 | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stderr.txt' |
| workbench build | success | 0 | 6021 | 'artifacts/workbench/beta-smoke/logs/workbench-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-build.stderr.txt' |
| workbench tauri tests | success | 0 | 2028 | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stderr.txt' |
| semantic release binaries | success | 0 | 1030 | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stderr.txt' |
| workbench release build | success | 0 | 318014 | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stderr.txt' |
| smoke diagnostics check | expected failure | 1 | 1020 | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stderr.txt' |
| smoke format check before write | expected failure | 1 | 1014 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stderr.txt' |
| smoke format write | success | 0 | 1014 | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stderr.txt' |
| smoke format check after write | success | 0 | 1013 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stderr.txt' |
| smoke compile | success | 0 | 1025 | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stderr.txt' |
| smoke verify | success | 0 | 1020 | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stderr.txt' |
| smoke disasm | success | 0 | 1032 | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stderr.txt' |
| smoke run | success | 0 | 1015 | 'artifacts/workbench/beta-smoke/logs/smoke-run.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-run.stderr.txt' |
| release bundle verify | success | 0 | 1015 | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stderr.txt' |

## Bundle Inventory

| Artifact | Size (bytes) | SHA256 |
| --- | ---: | --- |
| artifacts/workbench/beta-smoke/tauri-target/release/semantic-workbench-app.exe | 9077248 | '59e6840834c20d3d995c8011fc8314a725622dd58f57febb78e0695ce370b95f' |
| artifacts/workbench/beta-smoke/package/semantic-workbench-beta-portable.zip | 2895799 | 'e979dec3618fda6ca8425cbe283fb78b2818de7ee779be6fb3b26469bb2a042e' |
