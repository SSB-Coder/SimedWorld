param(
    [string]$Rules,
    [string]$Config,
    [string]$Load,
    [string]$Checkpoint = 'artifacts/world-checkpoint.json',
    [switch]$Paused
)

$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskOldEnvironment = @{}
foreach ($taskKey in @('PATH', 'INCLUDE', 'LIB', 'PYO3_PYTHON', 'VIRTUAL_ENV')) {
    $taskOldEnvironment[$taskKey] = [Environment]::GetEnvironmentVariable($taskKey, 'Process')
}
Push-Location -LiteralPath $taskRoot
try {
    . (Join-Path $PSScriptRoot 'windows_toolchain.ps1')
    Set-WorldBuildEnvironment -ProjectRoot $taskRoot
    & $env:PYO3_PYTHON scripts/audit_dependencies.py cargo Cargo.lock --output artifacts/setup/rust-workspace-audit.json
    if ($LASTEXITCODE -ne 0) { throw 'Workspace advisory check did not pass.' }
    & $env:PYO3_PYTHON scripts/plain_command.py -- cargo build --locked --offline -p world_viewer --jobs 2
    if ($LASTEXITCODE -ne 0) { throw 'Viewer build failed.' }
    $taskArguments = @('--checkpoint', $Checkpoint)
    foreach ($taskOption in @(@('--rules', $Rules), @('--config', $Config), @('--load', $Load))) {
        if ($taskOption[1]) { $taskArguments += $taskOption }
    }
    if ($Paused) { $taskArguments += '--paused' }
    & (Join-Path $taskRoot 'target/debug/world_viewer.exe') @taskArguments
    if ($LASTEXITCODE -ne 0) { throw 'Viewer exited with an error.' }
} finally {
    foreach ($taskKey in $taskOldEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($taskKey, $taskOldEnvironment[$taskKey], 'Process')
    }
    Pop-Location
}
