param([switch]$IncludeRust)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskOldEnvironment = @{}
foreach ($taskKey in @('PATH', 'INCLUDE', 'LIB', 'PYO3_PYTHON', 'VIRTUAL_ENV')) {
    $taskOldEnvironment[$taskKey] = [Environment]::GetEnvironmentVariable($taskKey, 'Process')
}
Push-Location -LiteralPath $taskRoot
try {
    $taskPython = Join-Path $taskRoot '.venv\Scripts\python.exe'
    & $taskPython scripts/verify_dependencies.py
    if ($LASTEXITCODE -ne 0) { throw 'Python dependency verification failed.' }
    & $taskPython -m pip check
    if ($LASTEXITCODE -ne 0) { throw 'Python package consistency check failed.' }
    if ($IncludeRust) {
        & $taskPython scripts/audit_dependencies.py cargo Cargo.lock --output artifacts/setup/rust-workspace-audit.json
        if ($LASTEXITCODE -ne 0) { throw 'Rust advisory review did not pass; compilation was not started.' }
        . (Join-Path $PSScriptRoot 'windows_toolchain.ps1')
        Set-WorldBuildEnvironment -ProjectRoot $taskRoot
        # Merge Cargo progress to avoid NativeCommandError in Windows PowerShell.
        & $env:ComSpec /d /c 'cargo check --locked --offline --workspace --all-features --jobs 2 2>&1'
        if ($LASTEXITCODE -ne 0) { throw 'Rust dependency build check failed.' }
    }
} finally {
    foreach ($taskKey in $taskOldEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($taskKey, $taskOldEnvironment[$taskKey], 'Process')
    }
    Pop-Location
}
