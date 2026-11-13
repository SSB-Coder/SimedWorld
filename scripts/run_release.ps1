param([ValidateSet('cpu','cuda')][string]$Device='cpu',[switch]$Headless,[int]$Decisions=300,[string]$Load,[string]$Python,[ValidateSet('sparse','ltc')][string]$Controller='sparse')
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$taskRun=Join-Path $taskRoot ('runs/' + [Guid]::NewGuid().ToString('N'))
$taskArguments=@((Join-Path $taskRoot 'examples/evolution_demo.py'),'--device',$Device,'--controller',$Controller,
    '--checkpoint',(Join-Path $taskRun 'checkpoint.json'),'--output',(Join-Path $taskRun 'result.json'),
    '--events',(Join-Path $taskRun 'events.jsonl'),'--stream',(Join-Path $taskRun 'observer.json'),
    '--commands',(Join-Path $taskRun 'commands'))
if ($Headless) { $taskArguments+=@('--decisions',$Decisions) } else { $taskArguments+=@('--interactive','--viewer','--viewer-executable',(Join-Path $taskRoot 'bin/world_viewer.exe')) }
if ($Load) { $taskArguments+=@('--resume',$Load) }
if (-not $Python) {$Python=Join-Path $taskRoot '.venv/Scripts/python.exe'}
& $Python -I @taskArguments
if ($LASTEXITCODE -ne 0) { throw 'Simulation failed; inspect the run directory.' }
