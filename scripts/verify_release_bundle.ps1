param(
    [string]$ManifestPath
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$requiredDirectories = @(
    "docs/architecture",
    "docs/spec",
    "tests/golden_snapshots/runtime",
    "tests/golden_snapshots/public_api"
)

$requiredFiles = @(
    "docs/release_artifact_model.md",
    "docs/roadmap/v1_readiness.md",
    "docs/roadmap/runtime_validation_policy.md",
    "docs/roadmap/release_bundle_checklist.md",
    "docs/roadmap/compatibility_statement.md",
    "tests/golden_semcode.rs",
    "tests/prometheus_runtime_matrix.rs",
    "tests/prometheus_runtime_goldens.rs",
    "tests/prometheus_runtime_negative_goldens.rs",
    "tests/prometheus_runtime_compat_matrix.rs",
    "tests/public_api_contracts.rs"
)

$missing = @()

foreach ($path in $requiredDirectories) {
    if (-not (Test-Path $path -PathType Container)) {
        $missing += $path
    }
}

foreach ($path in $requiredFiles) {
    if (-not (Test-Path $path -PathType Leaf)) {
        $missing += $path
    }
}

if ($missing.Count -gt 0) {
    $joined = ($missing | Sort-Object) -join ", "
    throw "release bundle verification failed; missing required paths: $joined"
}

$manifest = [ordered]@{
    generated_at = (Get-Date).ToString("yyyy-MM-ddTHH:mm:ssK")
    documentation_bundle = @(
        "docs/release_artifact_model.md",
        "docs/architecture",
        "docs/spec",
        "docs/roadmap/v1_readiness.md",
        "docs/roadmap/runtime_validation_policy.md",
        "docs/roadmap/release_bundle_checklist.md",
        "docs/roadmap/compatibility_statement.md"
    )
    validation_tests = @(
        "cargo test --workspace",
        "cargo test --test public_api_contracts",
        "cargo test --test golden_semcode",
        "cargo test --test prometheus_runtime_matrix",
        "cargo test --test prometheus_runtime_goldens",
        "cargo test --test prometheus_runtime_negative_goldens",
        "cargo test --test prometheus_runtime_compat_matrix"
    )
    snapshot_directories = @(
        "tests/golden_snapshots/public_api",
        "tests/golden_snapshots/runtime"
    )
    current_scope = "Semantic v1 narrow PROMETHEUS boundary"
}

if ($ManifestPath) {
    $dir = Split-Path -Parent $ManifestPath
    if ($dir) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    $manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $ManifestPath
    Write-Output "release bundle manifest written to $ManifestPath"
}

Write-Output "release bundle verification passed"
