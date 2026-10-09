<#
Focused unit tests for scripts/harness-check.ps1:
Tests positive path validation, negative boundaries, invariant protection,
transition logic, and safe return to ordinary SHF task execution.
#>
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptRoot
$CheckerPath = Join-Path $RepoRoot 'scripts/harness-check.ps1'

$failures = 0
$testsRun = 0

function Assert-Test([string]$Name, [bool]$Condition) {
    $script:testsRun++
    if ($Condition) {
        Write-Host "[test:PASS] $Name"
    } else {
        Write-Host "[test:FAIL] $Name"
        $script:failures++
    }
}

Write-Host "Running harness-check test suite..."

# Test 1: SelfTest passes
$proc = Start-Process -FilePath "pwsh" -ArgumentList "-NoProfile", "-File", "`"$CheckerPath`"", "-SelfTest" -NoNewWindow -Wait -PassThru
Assert-Test "Checker built-in selftest exits 0" ($proc.ExitCode -eq 0)

# Test 2: Invariant enforcement on ordinary task
# Verify that an ordinary task attempting to allow .github/** or scripts/harness-check.ps1 is rejected
$testTmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $testTmpDir -Force | Out-Null

try {
    $envOrdinaryWithForbiddenAllowed = @"
task:
  id: SHF-2-TEST
  title: "Ordinary SHF task"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - compiler/**
    - .github/**
  forbidden_paths:
    - bootstrap/**
    - "*.sm"
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $envFile = Join-Path $testTmpDir "env_ordinary_bad.yaml"
    Set-Content -LiteralPath $envFile -Value $envOrdinaryWithForbiddenAllowed -Encoding utf8

    $res = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$envFile'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'governance invariant path') { exit 42 }
            exit 1
        }
    "
    Assert-Test "Ordinary SHF task cannot allow governance invariant path in allowed_paths" ($LASTEXITCODE -eq 42)

    # Test 3: Governance task can allow governance paths
    $envGovernanceGood = @"
task:
  id: GOVERNANCE-TEST
  title: "Governance task"
  type: governance_migration
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - .harness/current.task.yaml
    - scripts/**
    - .github/**
    - docs/**
  forbidden_paths:
    - compiler/**
    - bootstrap/**
    - "*.sm"
authorization:
  workflow_changes: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $envGovFile = Join-Path $testTmpDir "env_gov.yaml"
    Set-Content -LiteralPath $envGovFile -Value $envGovernanceGood -Encoding utf8

    $resGov = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            `$cfg = Read-Envelope '$envGovFile'
            if (`$cfg.task.type -eq 'governance_migration') { exit 0 }
            exit 1
        } catch {
            exit 2
        }
    "
    Assert-Test "Governance task successfully allows governance paths when type is governance_migration" ($LASTEXITCODE -eq 0)

    # Test 4: Missing invariant forbidden paths in ordinary task fails
    $envOrdinaryMissingForbidden = @"
task:
  id: SHF-2-ORDINARY
  title: "Ordinary SHF task"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - compiler/**
    - tests/**
  forbidden_paths:
    - bootstrap/**
    - "*.sm"
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $envNoGovForbidFile = Join-Path $testTmpDir "env_no_gov_forbid.yaml"
    Set-Content -LiteralPath $envNoGovForbidFile -Value $envOrdinaryMissingForbidden -Encoding utf8

    $resNoGovForbid = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$envNoGovForbidFile'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'must forbid governance invariant') { exit 43 }
            exit 1
        }
    "
    Assert-Test "Ordinary task must explicitly forbid governance invariant paths" ($LASTEXITCODE -eq 43)

    # Test 5: Properly formed ordinary SHF task passes validation
    $envOrdinaryGood = @"
task:
  id: SHF-2-ORDINARY-VALID
  title: "Ordinary SHF task"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - compiler/**
    - tests/**
    - docs/**
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1
    - bootstrap/**
    - reference/**
    - "*.sm"
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $envOrdGoodFile = Join-Path $testTmpDir "env_ord_good.yaml"
    Set-Content -LiteralPath $envOrdGoodFile -Value $envOrdinaryGood -Encoding utf8

    $resOrdGood = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            `$cfg = Read-Envelope '$envOrdGoodFile'
            exit 0
        } catch {
            exit 1
        }
    "
    Assert-Test "Well-formed ordinary SHF task passes envelope validation" ($LASTEXITCODE -eq 0)

} finally {
    if (Test-Path -LiteralPath $testTmpDir) {
        Remove-Item -LiteralPath $testTmpDir -Recurse -Force
    }
}

Write-Host "Summary: $testsRun tests run, $script:failures failure(s)."
if ($script:failures -gt 0) { exit 1 } else { exit 0 }
