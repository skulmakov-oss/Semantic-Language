# Workbench Beta Smoke Report

- Generated: 2026-03-15T20:55:56.7653411Z
- Branch: codex/wb-bh-13-beta-packaging-smoke
- Commit: 4f42588a46249f4861b23fcc3a022c21f533365f
- Output root: artifacts/workbench/beta-smoke
- Workspace root: artifacts/workbench/beta-smoke/workspace
- Package format: portable-zip
- Portable zip: artifacts/workbench/beta-smoke/package/semantic-workbench-beta-portable.zip
- Release executable: apps/workbench/src-tauri/target/release/semantic-workbench-app.exe
- Launch smoke: passed (8 s window)

## Acceptance Coverage

- Packaged app launched from the extracted portable beta package.
- Smoke loop covered diagnostics, format check/write, compile, verify, disasm, run, and release bundle verification.
- Full command captures are preserved under 'artifacts/workbench/beta-smoke/logs/'.

## Step Summary

| Step | Expectation | Exit | Duration (ms) | Stdout | Stderr |
| --- | --- | ---: | ---: | --- | --- |
| workbench lint | success | 0 | 6031 | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-lint.stderr.txt' |
| workbench build | success | 0 | 6028 | 'artifacts/workbench/beta-smoke/logs/workbench-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-build.stderr.txt' |
| workbench tauri tests | success | 0 | 2030 | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-tauri-tests.stderr.txt' |
| semantic release binaries | success | 0 | 1039 | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/semantic-release-binaries.stderr.txt' |
| workbench release build | success | 0 | 79015 | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/workbench-release-build.stderr.txt' |
| smoke diagnostics check | expected failure | 1 | 1024 | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-diagnostics-check.stderr.txt' |
| smoke format check before write | expected failure | 1 | 1021 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-before-write.stderr.txt' |
| smoke format write | success | 0 | 1023 | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-write.stderr.txt' |
| smoke format check after write | success | 0 | 1024 | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-format-check-after-write.stderr.txt' |
| smoke compile | success | 0 | 1018 | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-compile.stderr.txt' |
| smoke verify | success | 0 | 1025 | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-verify.stderr.txt' |
| smoke disasm | success | 0 | 1013 | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-disasm.stderr.txt' |
| smoke run | success | 0 | 1026 | 'artifacts/workbench/beta-smoke/logs/smoke-run.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/smoke-run.stderr.txt' |
| release bundle verify | success | 0 | 1027 | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stdout.txt' | 'artifacts/workbench/beta-smoke/logs/release-bundle-verify.stderr.txt' |

## Bundle Inventory

| Artifact | Size (bytes) | SHA256 |
| --- | ---: | --- |
| apps/workbench/src-tauri/target/release/semantic-workbench-app.exe | 9077248 | '70126083f3868942d09f8855ba793db13396f2da96f6e8012274e769ab81c4ff' |
| artifacts/workbench/beta-smoke/package/semantic-workbench-beta-portable.zip | 2895680 | '73c67c6af525fd8b18132f0847121858f3220602c754c7f7a9038e4216177a75' |
