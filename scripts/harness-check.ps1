<#
Fail-closed Harness scope check for Semantic-Language.

  pwsh -File scripts/harness-check.ps1                 # local: staged + unstaged + untracked
  pwsh -File scripts/harness-check.ps1 -BaseRef <sha>  # committed <sha>...HEAD (+ local changes)
  pwsh -File scripts/harness-check.ps1 -SelfTest       # prove the guards fire

Path policy: forbidden match -> FAIL; else allowed match -> ok; else FAIL (outside scope).
Supported patterns (anything else is rejected as an envelope error):
  exact/path       exact, case-sensitive match of the Git path
  dir/**           any path strictly under dir/
  *.ext            any path at any depth whose name ends in .ext
Git paths are normalized to '/'. Exit 0 = pass, 1 = any violation or envelope error.
#>
param(
    [string]$BaseRef,
    [switch]$SelfTest,
    [switch]$RequireEnvelopeBase,  # PR mode: constraints.base_sha must equal -BaseRef
    # CI runs the BASE revision's checker from outside the tree against the PR checkout.
    [string]$RepoRoot = (Join-Path $PSScriptRoot '..'),
    [string]$TaskFile
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Read-Envelope([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "missing envelope: $Path" }
    $cfg = @{}
    $section = $null; $listKey = $null
    $n = 0
    foreach ($line in Get-Content -LiteralPath $Path -Encoding utf8) {
        $n++
        if ($line -match '[\x00-\x1f\x7f]') { throw "line ${n}: control character in envelope" }
        if ($line -match '^\s*(#.*)?$') { continue }
        if ($line -match '^([a-z_]+):\s*$') {
            $section = $Matches[1]; $listKey = $null
            if ($cfg.ContainsKey($section)) { throw "line ${n}: duplicate section '$section'" }
            $cfg[$section] = @{}
        } elseif ($line -match '^  ([a-z_]+):\s*(.*?)\s*$' -and $section) {
            $key = $Matches[1]; $value = $Matches[2]
            if ($cfg[$section].ContainsKey($key)) { throw "line ${n}: duplicate key '$section.$key'" }
            if ($value -eq '') {
                $listKey = $key; $cfg[$section][$key] = [System.Collections.Generic.List[string]]::new()
            } else {
                $listKey = $null
                if ($value -match '^"(.*)"$' -or $value -match "^'(.*)'$") { $value = $Matches[1] }
                $cfg[$section][$key] = $value
            }
        } elseif ($line -match '^    - (.+?)\s*$' -and $listKey) {
            $item = $Matches[1]
            if ($item -match '^"(.*)"$' -or $item -match "^'(.*)'$") { $item = $Matches[1] }
            $cfg[$section][$listKey].Add($item)
        } else {
            throw "line ${n}: unsupported envelope syntax: '$line'"
        }
    }
    Test-Envelope $cfg
    return $cfg
}

function Test-Envelope($cfg) {
    foreach ($s in 'task', 'intent', 'scope', 'authorization', 'constraints') {
        if (-not $cfg.ContainsKey($s) -or $cfg[$s].Count -eq 0) { throw "missing or empty section: $s" }
    }
    foreach ($k in 'task.id', 'task.title', 'task.type', 'task.mode', 'task.authorized_by',
                   'intent.summary', 'constraints.base_branch', 'constraints.base_sha', 'constraints.issue') {
        $s, $f = $k.Split('.')
        if (-not $cfg[$s].ContainsKey($f) -or $cfg[$s][$f] -isnot [string] -or $cfg[$s][$f] -eq '') {
            throw "missing required field: $k"
        }
    }
    if ($cfg.task.mode -cne 'active') { throw "task.mode must be 'active' (got '$($cfg.task.mode)')" }
    if ($cfg.constraints.base_sha -cnotmatch '^[0-9a-f]{40}$') { throw 'constraints.base_sha must be an exact 40-hex SHA' }
    if ($cfg.constraints.issue -notmatch '^[1-9][0-9]*$') { throw 'constraints.issue must be an issue number' }
    foreach ($k in 'allowed_paths', 'forbidden_paths') {
        if (-not $cfg.scope.ContainsKey($k) -or $cfg.scope[$k] -is [string]) { throw "scope.$k must be a list" }
    }
    if ($cfg.scope.allowed_paths.Count -eq 0) { throw 'scope.allowed_paths is empty' }
    foreach ($p in @($cfg.scope.allowed_paths) + @($cfg.scope.forbidden_paths)) { [void](Test-PatternSyntax $p) }
    # Invariant governance boundaries: non-governance tasks must never touch governance/CI, checker,
    # or the envelope file itself, and must explicitly keep them in forbidden_paths to ensure safe fail-closed return to ordinary SHF work.
    $isGovernanceTask = ($cfg.task.type -cin 'governance', 'governance_migration')
    if (-not $isGovernanceTask) {
        foreach ($p in $cfg.scope.allowed_paths) {
            # Stable ordinary engineering surfaces (Surface A):
            # compiler/**, tests/**, docs/**, README.md, CONTRIBUTING.md.
            $isSurfaceA = ($p -cin 'README.md', 'CONTRIBUTING.md') -or
                ($p -cin 'compiler/**', 'tests/**', 'docs/**') -or
                ($p.StartsWith('compiler/', [StringComparison]::Ordinal) -or
                 $p.StartsWith('tests/', [StringComparison]::Ordinal) -or
                 $p.StartsWith('docs/', [StringComparison]::Ordinal))
            if (-not $isSurfaceA) {
                throw "ordinary task allowed_paths must be restricted to stable engineering surface (compiler/**, tests/**, docs/**, README.md, CONTRIBUTING.md): '$p'"
            }
        }
        $forbidsGithub = @($cfg.scope.forbidden_paths | Where-Object { $_ -ceq '.github/**' }).Count -gt 0
        $forbidsChecker = @($cfg.scope.forbidden_paths | Where-Object { $_ -ceq 'scripts/harness-check.ps1' }).Count -gt 0
        $forbidsEnvelope = @($cfg.scope.forbidden_paths | Where-Object { $_ -ceq '.harness/current.task.yaml' }).Count -gt 0
        if (-not $forbidsGithub -or -not $forbidsChecker -or -not $forbidsEnvelope) {
            throw "ordinary task must explicitly forbid governance invariant paths: '.github/**', 'scripts/harness-check.ps1', and '.harness/current.task.yaml'"
        }
    }
    foreach ($k in $cfg.authorization.Keys) {
        if ($cfg.authorization[$k] -cnotin 'true', 'false') { throw "authorization.$k must be true or false" }
    }
}

function Test-PatternSyntax([string]$p) {
    if ($p -match '[\\\s#]' -or $p.StartsWith('/') -or $p -match '(^|/)\.\.?(/|$)') { throw "unsupported pattern: '$p'" }
    $body = if ($p.EndsWith('/**')) { $p.Substring(0, $p.Length - 3) }
            elseif ($p -match '^\*\.[A-Za-z0-9._-]+$') { '' }
            else { $p }
    if ($body -match '[*?\[\]{}]') { throw "unsupported pattern: '$p'" }
    return $true
}

function Test-PathMatch([string]$path, [string]$p) {
    if ($p.EndsWith('/**')) { return $path.StartsWith($p.Substring(0, $p.Length - 2), [StringComparison]::Ordinal) }
    if ($p.StartsWith('*.')) { return $path.EndsWith($p.Substring(1), [StringComparison]::Ordinal) }
    return [string]::Equals($path, $p, [StringComparison]::Ordinal)
}

function Get-Violations($cfg, [string[]]$paths) {
    foreach ($path in $paths) {
        # A literal backslash or control character could alias another path: never matchable.
        if ($path -match '[\\\x00-\x1f\x7f]') { "unsupported path name (backslash/control char): $($path -replace '[\x00-\x1f\x7f]', '?')"; continue }
        $f = @($cfg.scope.forbidden_paths | Where-Object { Test-PathMatch $path $_ })
        if ($f.Count) { "forbidden path changed: $path (matches '$($f[0])')"; continue }
        if (-not @($cfg.scope.allowed_paths | Where-Object { Test-PathMatch $path $_ }).Count) {
            "path outside allowed scope: $path"
        }
    }
}

function Invoke-Git {
    $out = & git -c core.quotepath=off @args
    if ($LASTEXITCODE -ne 0) { throw "git $args failed ($LASTEXITCODE)" }
    $out
}

# Raw stdout BYTES: PowerShell's native-command pipeline would split names containing newlines,
# and a StreamReader would eat a leading BOM. Split on byte 0, then decode each name as strict
# UTF-8 (no BOM handling; invalid bytes throw -> fail closed).
function Invoke-GitZ {
    $psi = [System.Diagnostics.ProcessStartInfo]::new('git')
    foreach ($a in @('-c', 'core.quotepath=off') + $args) { $psi.ArgumentList.Add($a) }
    $psi.RedirectStandardOutput = $true
    $psi.WorkingDirectory = (Get-Location).ProviderPath  # .NET cwd is not the PowerShell location
    $proc = [System.Diagnostics.Process]::Start($psi)
    $buf = [System.IO.MemoryStream]::new()
    $proc.StandardOutput.BaseStream.CopyTo($buf)
    $proc.WaitForExit()
    if ($proc.ExitCode -ne 0) { throw "git $args failed ($($proc.ExitCode))" }
    $utf8 = [System.Text.UTF8Encoding]::new($false, $true)
    $bytes = $buf.ToArray(); $start = 0
    for ($i = 0; $i -lt $bytes.Length; $i++) {
        if ($bytes[$i] -eq 0) { $utf8.GetString($bytes, $start, $i - $start); $start = $i + 1 }
    }
    if ($start -lt $bytes.Length) { $utf8.GetString($bytes, $start, $bytes.Length - $start) }
}

function Get-ChangedPaths([string]$base) {
    $paths = @()
    # -z: NUL-separated, never C-quoted, so unusual names are checked verbatim.
    $paths += Invoke-GitZ diff -z --name-only --no-renames --cached
    $paths += Invoke-GitZ diff -z --name-only --no-renames
    $paths += Invoke-GitZ ls-files -z --others --exclude-standard
    if ($base) {
        [void](Invoke-Git rev-parse --verify --quiet "$base^{commit}")
        $paths += Invoke-GitZ diff -z --name-only --no-renames "$base...HEAD"
    }
    # Git always separates with '/'; paths are taken verbatim, never rewritten.
    $paths | Where-Object { $_ } | Sort-Object -Unique -CaseSensitive
}

# A PR may not authorize its own payload. A scope change vs the BASE (already merged) envelope
# (task.id, allowed/forbidden paths, authorization flags) is a transition and must be
# envelope-only; the newly authorized work follows in a later PR. Updating only bookkeeping
# (constraints.*, title, summary) is not a transition.
function Test-SameList($a, $b) {
    $sa = @(@($a) | Sort-Object -CaseSensitive -Unique)
    $sb = @(@($b) | Sort-Object -CaseSensitive -Unique)
    if ($sa.Count -ne $sb.Count) { return $false }
    for ($i = 0; $i -lt $sa.Count; $i++) { if ($sa[$i] -cne $sb[$i]) { return $false } }
    return $true
}

function Test-SameScope($x, $y) {
    $kx = @($x.authorization.Keys | Sort-Object -CaseSensitive)
    $ky = @($y.authorization.Keys | Sort-Object -CaseSensitive)
    if (-not (Test-SameList $kx $ky)) { return $false }
    foreach ($k in $kx) {
        if ($x.authorization[$k] -cne $y.authorization[$k]) { return $false }
    }
    return ($x.task.id -ceq $y.task.id) -and
        ($x.task.type -ceq $y.task.type) -and
        (Test-SameList $x.scope.allowed_paths $y.scope.allowed_paths) -and
        (Test-SameList $x.scope.forbidden_paths $y.scope.forbidden_paths)
}

function Test-Transition($cfg, [string]$base, [string[]]$paths) {
    $text = & git show "${base}:.harness/current.task.yaml" 2>$null
    if ($LASTEXITCODE -ne 0) { Write-Host "[harness] no envelope at base $base (Harness bootstrap)"; return }
    $tmp = New-TemporaryFile
    try {
        Set-Content -LiteralPath $tmp -Value $text -Encoding utf8
        $old = try { Read-Envelope $tmp } catch { $null }
    } finally { Remove-Item -LiteralPath $tmp }
    if (-not $old) { 'base envelope exists but is unparseable (fail closed)'; return }
    if (Test-SameScope $old $cfg) { return }
    Write-Host '[harness] TRANSITION: envelope scope changed (requires owner authorization)'
    $added = @($cfg.scope.allowed_paths | Where-Object { $_ -cnotin $old.scope.allowed_paths })
    $removed = @($old.scope.forbidden_paths | Where-Object { $_ -cnotin $cfg.scope.forbidden_paths })
    if ($old.task.id -cne $cfg.task.id) {
        Write-Host "[harness] TRANSITION: task $($old.task.id) -> $($cfg.task.id) (requires owner authorization)"
    }
    if ($old.task.type -cne $cfg.task.type) {
        Write-Host "[harness] TRANSITION: task type $($old.task.type) -> $($cfg.task.type) (requires owner authorization)"
    }
    foreach ($a in $added) { Write-Host "[harness] TRANSITION: allowed_paths + $a" }
    foreach ($f in $removed) { Write-Host "[harness] TRANSITION: forbidden_paths - $f" }
    $paths | Where-Object { $_ -cne '.harness/current.task.yaml' } |
        ForEach-Object { "transition PR must be envelope-only; payload path: $_" }
}

function Invoke-SelfTest {
    $good = @'
task:
  id: T
  title: t
  type: governance
  mode: active
  authorized_by: owner
intent:
  summary: s
scope:
  allowed_paths:
    - AGENTS.md
    - scripts/**
    - docs/**
  forbidden_paths:
    - docs/frozen/**
    - "*.sm"
authorization:
  workflow_changes: false
constraints:
  issue: 1
  base_branch: main
  base_sha: 0123456789abcdef0123456789abcdef01234567
'@
    $tmp = New-TemporaryFile
    $script:failures = 0
    function Check-Env([string]$name, [string]$text, [bool]$ok) {
        Set-Content -LiteralPath $tmp -Value $text -Encoding utf8
        $got = try { [void](Read-Envelope $tmp); $true } catch { $false }
        if ($got -ne $ok) { Write-Host "[selftest:FAIL] $name"; $script:failures++ } else { Write-Host "[selftest] ok: $name" }
    }
    try {
        Check-Env 'valid envelope parses' $good $true
        Set-Content -LiteralPath $tmp -Value $good -Encoding utf8
        $e = Read-Envelope $tmp
        $cases = @(
            @('AGENTS.md', $true, 'allowed exact path'),
            @('scripts/harness-check.ps1', $true, 'allowed ** pattern'),
            @('scripts/a/b/c.ps1', $true, 'allowed ** nested'),
            @('README.md', $false, 'outside allowed path'),
            @('scriptsX/a', $false, 'dir/** does not match prefix sibling'),
            @('agents.md', $false, 'case-sensitive exact match'),
            @('bootstrap/x.sm', $false, 'forbidden *.ext outside allowed'),
            @('docs/frozen/a.md', $false, 'allowed + forbidden -> forbidden wins'),
            @('scripts/x.sm', $false, 'forbidden *.ext inside allowed dir -> forbidden wins'),
            @('scripts\payload', $false, 'literal backslash rejected'),
            @("AGENTS.md`nscripts/a", $false, 'embedded newline rejected')
        )
        foreach ($c in $cases) {
            $ok = -not @(Get-Violations $e @($c[0])).Count
            if ($ok -ne $c[1]) { Write-Host "[selftest:FAIL] $($c[2]): $($c[0])"; $script:failures++ } else { Write-Host "[selftest] ok: $($c[2])" }
        }
        Check-Env 'missing allowed_paths' ($good -replace '(?ms)  allowed_paths:.*?(?=  forbidden_paths:)', '') $false
        Check-Env 'empty allowed_paths' ($good -replace '(?m)^    - (AGENTS\.md|scripts/\*\*|docs/\*\*)\r?\n', '') $false
        Check-Env 'malformed line' ($good + "garbage line`n") $false
        Check-Env 'tab-indented item' ($good -replace '    - AGENTS.md', "`t- AGENTS.md") $false
        Check-Env 'missing envelope section' ($good -replace '(?ms)^constraints:.*', '') $false
        Check-Env 'task.mode not active' ($good -replace 'mode: active', 'mode: draft') $false
        Check-Env 'missing task.id' ($good -replace '(?m)^  id: T\r?\n', '') $false
        Check-Env 'malformed base_sha' ($good -replace 'base_sha: 0123', 'base_sha: main0123') $false
        Check-Env 'non-boolean authorization' ($good -replace 'workflow_changes: false', 'workflow_changes: maybe') $false
        Check-Env 'duplicate key' ($good -replace 'mode: active', "mode: active`n  mode: active") $false
        Check-Env 'unsupported glob' ($good -replace '- docs/\*\*', '- docs/*/x') $false
        Check-Env 'trailing comment on pattern' ($good -replace '- "\*\.sm"', '- *.sm # frozen') $false
        Check-Env 'control character in envelope' ($good -replace 'docs/\*\*', "scripts/a`0docs/**") $false
        Check-Env 'dot-dot pattern' ($good -replace '- AGENTS.md', '- ../AGENTS.md') $false
        Check-Env 'empty envelope' '' $false
        Remove-Item -LiteralPath $tmp
        $got = try { [void](Read-Envelope $tmp); $true } catch { $false }
        if ($got) { Write-Host '[selftest:FAIL] absent envelope'; $script:failures++ } else { Write-Host '[selftest] ok: absent envelope' }
    } finally {
        if (Test-Path -LiteralPath $tmp) { Remove-Item -LiteralPath $tmp }
    }
    if ($script:failures) { Write-Host "[harness:selftest] $script:failures failure(s)"; exit 1 }
    Write-Host '[harness:selftest] ok'
    exit 0
}

if ($SelfTest) { Invoke-SelfTest }

try {
    # Git paths below are repository-root relative regardless of the caller's directory.
    $RepoRoot = [System.IO.Path]::GetFullPath($RepoRoot, $PWD.Path)
    $TaskFile = if ($TaskFile) { [System.IO.Path]::GetFullPath($TaskFile, $PWD.Path) }
                else { Join-Path $RepoRoot '.harness/current.task.yaml' }
    Set-Location -LiteralPath $RepoRoot
    # The envelope must be a regular file, never a link that could redirect the read.
    if ((Get-Item -LiteralPath $TaskFile -Force -ErrorAction SilentlyContinue).LinkType) { throw "envelope is a link: $TaskFile" }
    $mode = "$(& git ls-files -s -- .harness/current.task.yaml)".Split(' ')[0]
    if ($mode -and $mode -cne '100644') { throw "envelope must be a regular file in git (mode $mode)" }
    $envelope = Read-Envelope $TaskFile
    Write-Host "[harness] task $($envelope.task.id) (issue #$($envelope.constraints.issue))"
    $mode = if ($BaseRef) { "committed $BaseRef...HEAD + working tree" } else { 'working tree (staged, unstaged, untracked)' }
    Write-Host "[harness] checking $mode"
    $paths = @(Get-ChangedPaths $BaseRef)
} catch {
    Write-Host "[harness:error] $($_.Exception.Message)"
    exit 1
}
# Check if this PR/change modifies the envelope itself.
# When an envelope-only transition PR runs, it touches strictly .harness/current.task.yaml.
# To prevent unauthorized tampering with protected envelopes, an envelope-only change is only
# exempted from Get-Violations if it is a genuine owner-authorized transition (detected via Test-Transition
# scope change vs BaseRef). Otherwise, if the envelope forbids .harness/current.task.yaml, Get-Violations
# fails closed.
$envelopeChanged = ($paths -contains '.harness/current.task.yaml')
$isEnvelopeOnly = ($paths.Count -eq 1 -and $paths[0] -ceq '.harness/current.task.yaml')
$isGenuineTransition = $false

if ($isEnvelopeOnly -and $BaseRef) {
    $text = & git show "${BaseRef}:.harness/current.task.yaml" 2>$null
    if ($LASTEXITCODE -eq 0) {
        $tmpBase = New-TemporaryFile
        try {
            Set-Content -LiteralPath $tmpBase -Value $text -Encoding utf8
            $oldBaseEnv = try { Read-Envelope $tmpBase } catch { $null }
        } finally { Remove-Item -LiteralPath $tmpBase }
        if ($oldBaseEnv -and -not (Test-SameScope $oldBaseEnv $envelope)) {
            $isGenuineTransition = $true
        }
    }
}

$payloadPaths = if ($isGenuineTransition) { @() } else { $paths }
$violations = @(Get-Violations $envelope $payloadPaths)
if ($BaseRef) {
    $full = "$(& git rev-parse --verify "$BaseRef^{commit}")".Trim()
    if ($RequireEnvelopeBase -and $envelopeChanged -and $envelope.constraints.base_sha -cne $full) {
        $violations += "constraints.base_sha $($envelope.constraints.base_sha) != PR base $full (stale envelope)"
    }
    $violations += @(Test-Transition $envelope $BaseRef $paths)
}
foreach ($v in $violations) { Write-Host "[harness:error] $v" }
if ($violations.Count) { exit 1 }
& git diff --check
if ($LASTEXITCODE -ne 0) { Write-Host '[harness:error] git diff --check failed'; exit 1 }
Write-Host "[harness] ok ($($paths.Count) changed path(s) within scope)"
