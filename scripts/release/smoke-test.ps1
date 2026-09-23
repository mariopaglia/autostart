param(
  [Parameter(Mandatory)][string]$Installer,
  [int]$AliveSeconds = 20
)

$ErrorActionPreference = 'Stop'
$installDir = Join-Path $env:LOCALAPPDATA 'AutoStart'
$logDir = Join-Path $env:LOCALAPPDATA 'com.mariopaglia.autostart\logs'

function Show-AppLogs {
  if (Test-Path $logDir) {
    Get-ChildItem $logDir -Filter *.log | ForEach-Object { Get-Content $_.FullName -Tail 50 }
  }
}

Write-Host "Installing $Installer silently"
$setup = Start-Process -FilePath $Installer -ArgumentList '/S' -Wait -PassThru
if ($setup.ExitCode -ne 0) { throw "Installer exited with code $($setup.ExitCode)" }

$app = Get-ChildItem $installDir -Filter *.exe | Where-Object Name -NotLike 'uninstall*' | Select-Object -First 1
if (-not $app) { throw "No app executable found in $installDir" }

Get-Process | Where-Object Path -EQ $app.FullName | Stop-Process -Force

Write-Host "Launching $($app.FullName)"
$process = Start-Process -FilePath $app.FullName -PassThru
Start-Sleep -Seconds $AliveSeconds
if ($process.HasExited) {
  Show-AppLogs
  throw "The app exited $AliveSeconds seconds after launch with code $($process.ExitCode)"
}
Write-Host "The app is still running after $AliveSeconds seconds"
Show-AppLogs

Stop-Process -Id $process.Id -Force
$uninstall = Start-Process -FilePath (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -Wait -PassThru
if ($uninstall.ExitCode -ne 0) { throw "Uninstaller exited with code $($uninstall.ExitCode)" }
Write-Host 'Smoke test passed'
