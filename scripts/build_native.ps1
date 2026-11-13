param([switch]$Wheel)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskOldEnvironment = @{}
foreach ($taskKey in @('PATH', 'INCLUDE', 'LIB', 'PYO3_PYTHON', 'VIRTUAL_ENV', 'PIP_NO_INDEX', 'UV_OFFLINE')) {
    $taskOldEnvironment[$taskKey] = [Environment]::GetEnvironmentVariable($taskKey, 'Process')
}
Push-Location -LiteralPath $taskRoot
try {
    . (Join-Path $PSScriptRoot 'windows_toolchain.ps1')
    Set-WorldBuildEnvironment -ProjectRoot $taskRoot
    & $env:PYO3_PYTHON scripts/audit_dependencies.py cargo Cargo.lock --output artifacts/setup/rust-workspace-audit.json
    if ($LASTEXITCODE -ne 0) { throw 'Workspace advisory check did not pass.' }
    $env:PIP_NO_INDEX = '1'
    $env:UV_OFFLINE = 'true'
    # Merge normal Cargo stderr progress before Windows PowerShell handles it.
    if ($Wheel) {
        & $env:PYO3_PYTHON scripts/plain_command.py -- .venv\Scripts\maturin.exe build --locked --offline --jobs 2 --interpreter .venv\Scripts\python.exe --out artifacts\wheels
    } else {
        & $env:PYO3_PYTHON scripts/plain_command.py -- .venv\Scripts\maturin.exe develop --locked --offline --jobs 2
    }
    if ($LASTEXITCODE -ne 0) { throw 'Native package build/install failed.' }
} finally {
    foreach ($taskKey in $taskOldEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($taskKey, $taskOldEnvironment[$taskKey], 'Process')
    }
    Pop-Location
}
