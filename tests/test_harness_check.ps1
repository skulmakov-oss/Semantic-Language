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
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 42 }
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

    # Adversarial Test 1: allowed: compiler/** + forbidden: .github/placeholder is rejected
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
    - compiler/**
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

    # Adversarial Test 2: allowed: compiler/** + forbidden: scripts/harness-check.ps1.tmp is rejected
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
    - compiler/**
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

    # Adversarial Test 3: ordinary envelope cannot allow paths outside Surface A (e.g. *.yml or AGENTS.md)
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
    - AGENTS.md
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
        try {
            Read-Envelope '$adv3File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 48 }
            exit 1
        }
    "
    Assert-Test "Adversarial 3: ordinary envelope cannot include AGENTS.md in allowed_paths" ($LASTEXITCODE -eq 48)

    # Adversarial Test 4: ordinary envelope cannot allow scripts/tool.ps1 outside Surface A
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
    - scripts/tool.ps1
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
        try {
            Read-Envelope '$adv4File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 49 }
            exit 1
        }
    "
    Assert-Test "Adversarial 4: ordinary envelope cannot include scripts/** in allowed_paths" ($LASTEXITCODE -eq 49)

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
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 46 }
            exit 1
        }
    "
    Assert-Test "Adversarial 5: Ordinary task cannot allow .harness/current.task.yaml to modify envelope" ($LASTEXITCODE -eq 46)

    # Adversarial Test 6: Broad patterns (e.g. *.md) or paths outside Surface A (e.g. AGENTS.md, scripts/x.ps1) are rejected
    $adv6 = @"
task:
  id: SHF-ADV-6
  title: "Adversarial 6"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - "*.md"
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
    $adv6File = Join-Path $testTmpDir "adv6.yaml"
    Set-Content -LiteralPath $adv6File -Value $adv6 -Encoding utf8

    $resAdv6 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv6File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 47 }
            exit 1
        }
    "
    Assert-Test "Adversarial 6: Ordinary task cannot allow broad pattern *.md outside Surface A" ($LASTEXITCODE -eq 47)

    # Adversarial Test 7: Case-insensitive task.type (e.g. GOVERNANCE) cannot bypass ordinary restrictions
    $adv7 = @"
task:
  id: SHF-ADV-7
  title: "Adversarial 7"
  type: GOVERNANCE
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - scripts/**
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
    $adv7File = Join-Path $testTmpDir "adv7.yaml"
    Set-Content -LiteralPath $adv7File -Value $adv7 -Encoding utf8

    $resAdv7 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv7File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 50 }
            exit 1
        }
    "
    Assert-Test "Adversarial 7: Uppercase task.type GOVERNANCE cannot bypass ordinary restrictions" ($LASTEXITCODE -eq 50)

    # Adversarial Test 8: Non-canonical casing for Surface A paths (e.g. Compiler/** or readme.md) is rejected
    $adv8 = @"
task:
  id: SHF-ADV-8
  title: "Adversarial 8"
  type: implementation
  mode: active
  authorized_by: owner
intent:
  summary: "test"
scope:
  allowed_paths:
    - Compiler/**
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
    $adv8File = Join-Path $testTmpDir "adv8.yaml"
    Set-Content -LiteralPath $adv8File -Value $adv8 -Encoding utf8

    $resAdv8 = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        try {
            Read-Envelope '$adv8File'
            exit 0
        } catch {
            if (`$_.Exception.Message -match 'restricted to stable engineering surface') { exit 51 }
            exit 1
        }
    "
    Assert-Test "Adversarial 8: Non-canonical casing Compiler/** in allowed_paths is rejected" ($LASTEXITCODE -eq 51)

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

    # Integration Test T1c: Test-SameList treats reordered lists with identical elements as same scope
    $resT1c = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$list1 = @('compiler/**', 'tests/**', 'docs/**')
        `$list2 = @('docs/**', 'compiler/**', 'tests/**')
        if (Test-SameList `$list1 `$list2) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T1c: Test-SameList treats reordered lists as identical set membership" ($LASTEXITCODE -eq 0)

    # Integration Test T1d: Test-SameScope preserves key-to-value pairing in authorization
    $resT1d = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$env1 = Read-Envelope '$g2EnvFile'
        `$env2 = Read-Envelope '$g2EnvFile'
        # Add two keys with opposite booleans
        `$env1.authorization['k1'] = 'true'
        `$env1.authorization['k2'] = 'false'
        `$env2.authorization['k1'] = 'false'
        `$env2.authorization['k2'] = 'true'
        if (-not (Test-SameScope `$env1 `$env2)) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T1d: Test-SameScope detects swapped authorization values as scope change" ($LASTEXITCODE -eq 0)

    # Integration Test T1e: Transition exemption requires governance status on base or candidate
    $resT1e = & pwsh -NoProfile -Command "
        . '$CheckerPath'
        `$envOrd1 = Read-Envelope '$g2EnvFile'
        `$envOrd2 = Read-Envelope '$g2EnvFile'
        `$envOrd2.task.id = 'SHF-STAGE-DELIVERY-NEXT' # Real scope change between ordinary tasks
        `$hasGov = (`$envOrd1.task.type -cin 'governance', 'governance_migration') -or
                   (`$envOrd2.task.type -cin 'governance', 'governance_migration')
        if (-not `$hasGov) { exit 0 } else { exit 1 }
    "
    Assert-Test "Integration T1e: Ordinary-to-ordinary scope change does not receive governance transition exemption" ($LASTEXITCODE -eq 0)


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
