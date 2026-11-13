param([string]$Python = 'py', [switch]$Offline, [string]$Wheelhouse)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskManifest = Get-Content -LiteralPath (Join-Path $taskRoot 'manifest.json') -Raw | ConvertFrom-Json
foreach ($taskFile in $taskManifest.files.PSObject.Properties) {
    $taskPath = Join-Path $taskRoot $taskFile.Name
    if ((Get-FileHash -LiteralPath $taskPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskFile.Value) {
        throw "Release checksum mismatch: $($taskFile.Name)"
    }
}
$taskLauncherArguments = @()
if ((Split-Path -Leaf $Python) -match '^py(\.exe)?$') { $taskLauncherArguments += '-3.13' }
& $Python @taskLauncherArguments -m venv (Join-Path $taskRoot '.venv')
if ($LASTEXITCODE -ne 0) { throw 'CPython 3.13 x64 is required.' }
$taskPython = Join-Path $taskRoot '.venv/Scripts/python.exe'
$taskArguments = @('-m', 'pip', 'install', '--require-hashes', '-r', (Join-Path $taskRoot 'dependencies/python-win-py313.lock'))
if ($Offline) {
    if (-not $Wheelhouse) { throw 'Offline installation requires a wheelhouse.' }
    $taskArguments += @('--no-index', '--find-links', $Wheelhouse)
}
& $taskPython @taskArguments
if ($LASTEXITCODE -ne 0) { throw 'Pinned Python dependency installation failed.' }
$taskWheels = @(Get-ChildItem -LiteralPath (Join-Path $taskRoot 'wheels') -Filter 'world_microcosm-*.whl')
if ($taskWheels.Count -ne 1) { throw 'Exactly one native wheel is required.' }
$taskWheel = $taskWheels[0]
& $taskPython -m pip install --no-deps --no-index $taskWheel.FullName
if ($LASTEXITCODE -ne 0) { throw 'Native wheel installation failed.' }
& $taskPython -m pip check
if ($LASTEXITCODE -ne 0) { throw 'Installed dependencies are inconsistent.' }
