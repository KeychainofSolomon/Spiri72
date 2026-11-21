# Tests the updater on a made-up install, from a local release folder. Run by
# CI on Windows: powershell -File update/probatio.ps1 <release folder> <built exe>
param([string]$Fons, [string]$Exe)
$ErrorActionPreference = "Stop"
$updater = Join-Path $PSScriptRoot "update.ps1"
$home2 = Join-Path ([IO.Path]::GetTempPath()) ("spiri72-probatio-" + [Guid]::NewGuid().ToString("N"))
function Check($ok, $what) { if (-not $ok) { Write-Host "FAILED: $what"; exit 1 } else { Write-Host "ok: $what" } }

# a made-up install: an old program and the Operator's own files
New-Item -ItemType Directory -Path $home2 | Out-Null
Set-Content (Join-Path $home2 "spiri72.exe") "old program"
Set-Content (Join-Path $home2 "spiri72.ini") "model = C:\models\x.gguf"
New-Item -ItemType Directory -Path (Join-Path $home2 "spiri72.volumina") | Out-Null
Set-Content (Join-Path $home2 "spiri72.volumina\behave.claves") "# claves: love"
New-Item -ItemType Directory -Path (Join-Path $home2 "spiri72.liber") | Out-Null
Set-Content (Join-Path $home2 "spiri72.liber\34.liber") "Furfur's book"
$mine = Get-ChildItem -Recurse -File $home2 | Where-Object { $_.Name -ne "spiri72.exe" } | ForEach-Object { @{ Path = $_.FullName; Hash = (Get-FileHash $_.FullName).Hash } }

& powershell -NoProfile -ExecutionPolicy Bypass -File $updater -Fons $Fons -Domus $home2
Check ($LASTEXITCODE -eq 0) "the update runs"
$want = (Get-FileHash $Exe).Hash
Check ((Get-FileHash (Join-Path $home2 "spiri72.exe")).Hash -eq $want) "the new program is in place"
Check ((Get-Content (Join-Path $home2 "spiri72.exe.old") -Raw).Trim() -eq "old program") "the old program is kept"
Check ((Get-Content (Join-Path $home2 "spiri72.versio") -Raw).Trim() -eq (Get-Content (Join-Path $Fons "VERSIO.txt") -Raw).Trim()) "the build is recorded"
foreach ($f in $mine) { Check ((Get-FileHash $f.Path).Hash -eq $f.Hash) "untouched: $($f.Path)" }
Check (Test-Path (Join-Path $home2 "update.cmd")) "the updater itself is updated"

# a second run finds it up to date and changes nothing
$before = (Get-FileHash (Join-Path $home2 "spiri72.exe.old")).Hash
& powershell -NoProfile -ExecutionPolicy Bypass -File $updater -Fons $Fons -Domus $home2
Check ($LASTEXITCODE -eq 0) "a second run passes"
Check ((Get-FileHash (Join-Path $home2 "spiri72.exe.old")).Hash -eq $before) "a second run changes nothing"

# a damaged download is refused, and nothing changes
$bad = Join-Path ([IO.Path]::GetTempPath()) ("spiri72-bad-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $bad | Out-Null
Copy-Item (Join-Path $Fons "SHA256SUMS.txt") $bad
$bytes = [IO.File]::ReadAllBytes((Join-Path $Fons "spiri72-windows.zip"))
$bytes[$bytes.Length - 40] = $bytes[$bytes.Length - 40] -bxor 0xff
[IO.File]::WriteAllBytes((Join-Path $bad "spiri72-windows.zip"), $bytes)
Remove-Item (Join-Path $home2 "spiri72.versio")
$exeBefore = (Get-FileHash (Join-Path $home2 "spiri72.exe")).Hash
& powershell -NoProfile -ExecutionPolicy Bypass -File $updater -Fons $bad -Domus $home2
Check ($LASTEXITCODE -eq 5) "a damaged download is refused"
Check ((Get-FileHash (Join-Path $home2 "spiri72.exe")).Hash -eq $exeBefore) "a refused update changes nothing"

Remove-Item -Recurse -Force $home2, $bad
Write-Host "The updater passed."
