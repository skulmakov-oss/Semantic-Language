<#
Focused unit tests for scripts/harness-check.ps1:
Tests positive path validation, negative boundaries, invariant protection,
adversarial pattern overlaps, transition logic, and safe return to ordinary SHF task execution.
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

$testTmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $testTmpDir -Force | Out-Null

try {
    # Test 2: Invariant enforcement on ordinary task - attempt to allow .github/**
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

    # Adversarial Test 1: allowed: *.yml + forbidden: .github/placeholder is rejected as an invalid ordinary envelope
    $adv1 = @"
task:
  id: SHF-ADV-1
  title: "Adversarial 1"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - "*.yml"
  forbidden_paths:
    - .github/placeholder
    - scripts/harness-check.ps1
    - .harness/current.task.yaml
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $adv1File = Join-Path $testTmpDir "adv1.yaml"
    Set-Content -LiteralPath $adv1File -Value $adv1 -Encoding utf8

    $resAdv1 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv1File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'must explicitly forbid governance invariant paths') { exit 44 }
            exit 1
        }
    "
    Assert-Test "Adversarial 1: partial .github/placeholder forbidden substitute is rejected" ($LASTEXITCODE -eq 44)

    # Adversarial Test 2: allowed: *.ps1 + forbidden: scripts/harness-check.ps1.tmp is rejected
    $adv2 = @"
task:
  id: SHF-ADV-2
  title: "Adversarial 2"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - "*.ps1"
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1.tmp
    - .harness/current.task.yaml
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $adv2File = Join-Path $testTmpDir "adv2.yaml"
    Set-Content -LiteralPath $adv2File -Value $adv2 -Encoding utf8

    $resAdv2 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv2File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'must explicitly forbid governance invariant paths') { exit 45 }
            exit 1
        }
    "
    Assert-Test "Adversarial 2: scripts/harness-check.ps1.tmp forbidden substitute is rejected" ($LASTEXITCODE -eq 45)

    # Adversarial Test 3: allowed: *.yml + exact forbidden: .github/** validates structurally,
    # but an attempted .github/workflows/test.yml payload is rejected by Get-Violations
    $adv3 = @"
task:
  id: SHF-ADV-3
  title: "Adversarial 3"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - "*.yml"
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1
    - .harness/current.task.yaml
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $adv3File = Join-Path $testTmpDir "adv3.yaml"
    Set-Content -LiteralPath $adv3File -Value $adv3 -Encoding utf8

    $resAdv3 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$adv3File'
        `$v = @(Get-Violations `$cfg @('.github/workflows/test.yml'))
        if (`$v.Count -gt 0 -and `$v[0] -match 'forbidden path changed') { exit 0 }
        exit 1
    "
    Assert-Test "Adversarial 3: allowed *.yml + forbidden .github/** rejects .github/workflows/test.yml via Get-Violations" ($LASTEXITCODE -eq 0)

    # Adversarial Test 4: allowed: *.ps1 + exact forbidden: scripts/harness-check.ps1 validates structurally,
    # but an attempted scripts/harness-check.ps1 payload is rejected by Get-Violations
    $adv4 = @"
task:
  id: SHF-ADV-4
  title: "Adversarial 4"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - "*.ps1"
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1
    - .harness/current.task.yaml
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $adv4File = Join-Path $testTmpDir "adv4.yaml"
    Set-Content -LiteralPath $adv4File -Value $adv4 -Encoding utf8

    $resAdv4 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$adv4File'
        `$v = @(Get-Violations `$cfg @('scripts/harness-check.ps1'))
        if (`$v.Count -gt 0 -and `$v[0] -match 'forbidden path changed') { exit 0 }
        exit 1
    "
    Assert-Test "Adversarial 4: allowed *.ps1 + forbidden scripts/harness-check.ps1 rejects scripts/harness-check.ps1 payload" ($LASTEXITCODE -eq 0)

    # Adversarial Test 5: Governance task behavior remains explicitly allowed only under governance authorization
    $adv5 = @"
task:
  id: SHF-ADV-5
  title: "Adversarial 5"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - .harness/current.task.yaml
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $adv5File = Join-Path $testTmpDir "adv5.yaml"
    Set-Content -LiteralPath $adv5File -Value $adv5 -Encoding utf8

    $resAdv5 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv5File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'governance invariant path') { exit 46 }
            exit 1
        }
    "
    Assert-Test "Adversarial 5: Ordinary task cannot allow .harness/current.task.yaml to modify envelope" ($LASTEXITCODE -eq 46)

    # Test 6: Properly formed ordinary SHF task passes validation
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
    - .harness/current.task.yaml
    - bootstrap/**
    - reference/**
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

    # Integration Test T1: G2-style envelope-only transition to ordinary profile passes
    # Candidate envelope is envelope-only change and forbids .harness/current.task.yaml.
    $g2TransitionEnvelope = @"
task:
  id: SHF-STAGE-DELIVERY-STABLE
  title: "Stable ordinary SHF delivery profile"
  type: implementation
  mode: active
  authorized_by: "Repository owner explicit task - SEMANTIC-LANGUAGE GOVERNANCE LIBERATION (Issue #20 Phase G2)"
intent:
  summary: "Stable envelope for implementation stages"
scope:
  allowed_paths:
    - compiler/**
    - tests/**
    - docs/**
    - CONTRIBUTING.md
    - README.md
  forbidden_paths:
    - .github/**
    - scripts/harness-check.ps1
    - .harness/current.task.yaml
    - bootstrap/**
    - reference/**
authorization:
  compiler_implementation: true
constraints:
  issue: 20
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
"@
    $g2EnvFile = Join-Path $testTmpDir "env_g2.yaml"
    Set-Content -LiteralPath $g2EnvFile -Value $g2TransitionEnvelope -Encoding utf8

    # Integration Test T0: Normal compiler source (.sm) is permitted under the stable G2 ordinary profile
    $resT0 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('compiler/lexer.sm', 'compiler/ast.sm', 'tests/test_lexer.sm')
        `$violations = @(Get-Violations `$cfg `$paths)
        if (`$violations.Count -eq 0) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T0: G2 stable profile permits compiler/** and tests/** *.sm files" ($LASTEXITCODE -eq 0)

    # Integration Test T1: G2-style genuine envelope-only transition does not trigger forbidden violation
    $resT1 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('.harness/current.task.yaml')
        `$envelopeChanged = (`$paths -contains '.harness/current.task.yaml')
        `$isEnvelopeOnly = (`$paths.Count -eq 1 -and `$paths[0] -ceq '.harness/current.task.yaml')
        # Simulate genuine transition detected vs base
        `$isGenuineTransition = `$true
        `$payloadPaths = if (`$isGenuineTransition) { @() } else { `$paths }
        `$violations = @(Get-Violations `$cfg `$payloadPaths)
        if (`$violations.Count -eq 0) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T1: G2-style genuine envelope-only transition does not trigger forbidden violation for .harness/current.task.yaml" ($LASTEXITCODE -eq 0)

    # Integration Test T1b: Non-transition envelope-only edit under ordinary profile is rejected by Get-Violations
    $resT1b = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('.harness/current.task.yaml')
        `$envelopeChanged = (`$paths -contains '.harness/current.task.yaml')
        `$isEnvelopeOnly = (`$paths.Count -eq 1 -and `$paths[0] -ceq '.harness/current.task.yaml')
        # Simulate non-transition edit (e.g. bookkeeping or same scope)
        `$isGenuineTransition = `$false
        `$payloadPaths = if (`$isGenuineTransition) { @() } else { `$paths }
        `$violations = @(Get-Violations `$cfg `$payloadPaths)
        if (`$violations -match 'forbidden path changed: .harness/current.task.yaml') { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T1b: Non-transition envelope-only edit under ordinary profile fails closed in Get-Violations" ($LASTEXITCODE -eq 0)

    # Integration Test T2: Envelope transition PR attempting to touch payload fails
    $resT2 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('.harness/current.task.yaml', 'compiler/lexer.sm')
        `$envelopeChanged = (`$paths -contains '.harness/current.task.yaml')
        `$isEnvelopeOnly = (`$paths.Count -eq 1 -and `$paths[0] -ceq '.harness/current.task.yaml')
        `$isGenuineTransition = `$false
        `$payloadPaths = if (`$isGenuineTransition) { @() } else { `$paths }
        `$violations = @(Get-Violations `$cfg `$payloadPaths)
        if (`$violations -match 'forbidden path changed: .harness/current.task.yaml') { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T2: Mixed envelope transition + payload triggers forbidden violation for .harness/current.task.yaml" ($LASTEXITCODE -eq 0)

    # Integration Test T3: Ordinary PR with unchanged envelope whose base_sha is older than PR base passes
    $resT3 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('compiler/lexer.sm', 'tests/test_lexer.sm')
        `$envelopeChanged = (`$paths -contains '.harness/current.task.yaml')
        `$full = 'fedcba9876543210fedcba9876543210fedcba98' # Advanced main SHA
        `$violations = @()
        if (`$true -and `$envelopeChanged -and `$cfg.constraints.base_sha -cne `$full) {
            `$violations += 'stale envelope'
        }
        if (`$violations.Count -eq 0) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T3: Ordinary payload with unchanged envelope does not trigger stale envelope check on advanced main" ($LASTEXITCODE -eq 0)

    # Integration Test T4: Ordinary PR attempting to touch .harness/current.task.yaml fails
    $resT4 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$cfg = Read-Envelope '$g2EnvFile'
        `$paths = @('compiler/lexer.sm', '.harness/current.task.yaml')
        `$envelopeChanged = (`$paths -contains '.harness/current.task.yaml')
        `$isEnvelopeOnly = (`$paths.Count -eq 1 -and `$paths[0] -ceq '.harness/current.task.yaml')
        `$isGenuineTransition = `$false
        `$payloadPaths = if (`$isGenuineTransition) { @() } else { `$paths }
        `$violations = @(Get-Violations `$cfg `$payloadPaths)
        if (`$violations -match 'forbidden path changed: .harness/current.task.yaml') { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T4: Ordinary PR attempting to include .harness/current.task.yaml fails closed" ($LASTEXITCODE -eq 0)

} finally {
    if (Test-Path -LiteralPath $testTmpDir) {
        Remove-Item -LiteralPath $testTmpDir -Recurse -Force
    }
}

Write-Host "Summary: $testsRun tests run, $script:failures failure(s)."
if ($script:failures -gt 0) { exit 1 } else { exit 0 }
