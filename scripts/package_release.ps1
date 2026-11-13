param([Parameter(Mandatory=$true)][string]$Output)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$taskOldEnvironment=@{}
foreach ($taskKey in @('PATH','INCLUDE','LIB','PYO3_PYTHON','VIRTUAL_ENV')) {
    $taskOldEnvironment[$taskKey]=[Environment]::GetEnvironmentVariable($taskKey,'Process')
}
Push-Location -LiteralPath $taskRoot
try {
    . (Join-Path $PSScriptRoot 'windows_toolchain.ps1')
    Set-WorldBuildEnvironment -ProjectRoot $taskRoot
    & $env:PYO3_PYTHON scripts/plain_command.py -- cargo test --locked --offline --workspace --jobs 2
    if ($LASTEXITCODE -ne 0) { throw 'Rust release checks failed.' }
    & $env:PYO3_PYTHON scripts/plain_command.py -- $env:PYO3_PYTHON -m unittest discover -s tests
    if ($LASTEXITCODE -ne 0) { throw 'Python release checks failed.' }
    & $env:PYO3_PYTHON scripts/plain_command.py -- .venv/Scripts/maturin.exe build --locked --offline --jobs 2 --interpreter $env:PYO3_PYTHON --out artifacts/wheels
    if ($LASTEXITCODE -ne 0) { throw 'Wheel build failed.' }
    & $env:PYO3_PYTHON scripts/plain_command.py -- cargo build --locked --offline -p world_viewer --jobs 2
    if ($LASTEXITCODE -ne 0) { throw 'Viewer build failed; close running viewers before rebuilding.' }
    & $env:PYO3_PYTHON scripts/package_release.py --output $Output
    if ($LASTEXITCODE -ne 0) { throw 'Release assembly failed.' }
} finally {
    foreach ($taskKey in $taskOldEnvironment.Keys) {[Environment]::SetEnvironmentVariable($taskKey,$taskOldEnvironment[$taskKey],'Process')}
    Pop-Location
}
