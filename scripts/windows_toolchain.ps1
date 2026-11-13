function Set-WorldBuildEnvironment {
    param([Parameter(Mandatory = $true)][string]$ProjectRoot)

    $taskPython = Join-Path $ProjectRoot '.venv\Scripts\python.exe'
    if (-not (Test-Path -LiteralPath $taskPython)) {
        throw 'Project .venv is missing; see DEPENDENCIES.md.'
    }
    $taskMsvcRoot = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC'
    $taskMsvc = Get-ChildItem -LiteralPath $taskMsvcRoot -Directory | Where-Object {
        Test-Path -LiteralPath (Join-Path $_.FullName 'bin\Hostx64\x64\link.exe')
    } | Sort-Object Name -Descending | Select-Object -First 1
    $taskSdkRoot = 'C:\Program Files (x86)\Windows Kits\10'
    $taskSdkLib = Join-Path $taskSdkRoot 'Lib'
    if (-not (Test-Path -LiteralPath $taskSdkLib)) {
        throw 'Windows SDK not detected; see DEPENDENCIES.md.'
    }
    $taskSdk = Get-ChildItem -LiteralPath $taskSdkLib -Directory | Where-Object {
        (Test-Path -LiteralPath (Join-Path $_.FullName 'um\x64\kernel32.lib')) -and
        (Test-Path -LiteralPath (Join-Path $_.FullName 'ucrt\x64\ucrt.lib'))
    } | Sort-Object Name -Descending | Select-Object -First 1
    if ($null -eq $taskMsvc -or $null -eq $taskSdk) {
        throw 'MSVC/x64 Windows SDK components are incomplete; see DEPENDENCIES.md.'
    }
    $taskSdkInclude = Join-Path $taskSdkRoot ('Include\' + $taskSdk.Name)
    $env:PATH = (Join-Path $ProjectRoot '.venv\Scripts') + ';' + (Join-Path $taskMsvc.FullName 'bin\Hostx64\x64') + ';' + $env:PATH
    $env:LIB = (Join-Path $taskMsvc.FullName 'lib\x64') + ';' + (Join-Path $taskSdk.FullName 'um\x64') + ';' + (Join-Path $taskSdk.FullName 'ucrt\x64')
    $env:INCLUDE = (Join-Path $taskMsvc.FullName 'include') + ';' + (Join-Path $taskSdkInclude 'ucrt') + ';' + (Join-Path $taskSdkInclude 'shared') + ';' + (Join-Path $taskSdkInclude 'um')
    $env:PYO3_PYTHON = $taskPython
    $env:VIRTUAL_ENV = Join-Path $ProjectRoot '.venv'
}
