param(
    [ValidateSet('cpu', 'cuda')][string]$Device = 'cpu',
    [ValidateSet('sparse','ltc')][string]$Controller='sparse',
    [string]$Load,
    [string]$RunDirectory,
    [ValidateSet('food_web','circle','hunting')][string]$Preset='food_web',
    [switch]$Paused,
    [Nullable[UInt64]]$RunForTicks,
    [string]$Capture
)

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
    & $env:PYO3_PYTHON scripts/plain_command.py -- .venv\Scripts\maturin.exe develop --locked --offline --jobs 2
    if ($LASTEXITCODE -ne 0) { throw 'Native package build failed.' }
    & $env:PYO3_PYTHON scripts/plain_command.py -- cargo build --locked --offline -p world_viewer --jobs 2
    if ($LASTEXITCODE -ne 0) { throw 'Viewer build failed.' }
    $taskArguments = @('examples/evolution_demo.py', '--device', $Device, '--controller', $Controller, '--interactive', '--viewer')
    $taskArguments += @('--rules', "configs/$($Preset)_rules.json", '--config', "configs/$($Preset)_habitat.json")
    if (-not $RunDirectory) { $RunDirectory = 'artifacts/runs/' + [Guid]::NewGuid().ToString('N') }
    $taskRunPath = if ([System.IO.Path]::IsPathRooted($RunDirectory)) { $RunDirectory } else { Join-Path $taskRoot $RunDirectory }
    $taskRunDirectory = [System.IO.Path]::GetFullPath($taskRunPath)
    foreach ($taskOutput in @(
        @('--checkpoint', 'evolution-checkpoint.json'), @('--output', 'evolution-run.json'),
        @('--events', 'evolution-events.jsonl'), @('--stream', 'evolution-observer.json'),
        @('--commands', 'evolution-commands')
    )) {
        $taskArguments += @($taskOutput[0], (Join-Path $taskRunDirectory $taskOutput[1]))
    }
    if ($Load) { $taskArguments += @('--resume', $Load) }
    if ($Paused) { $taskArguments += '--paused' }
    if ($null -ne $RunForTicks) { $taskArguments += @('--viewer-run-for-ticks', "$RunForTicks") }
    if ($Capture) { $taskArguments += @('--viewer-capture', $Capture) }
    & $env:PYO3_PYTHON @taskArguments
    if ($LASTEXITCODE -ne 0) { throw 'Evolution runtime failed.' }
} finally {
    foreach ($taskKey in $taskOldEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($taskKey, $taskOldEnvironment[$taskKey], 'Process')
    }
    Pop-Location
}
