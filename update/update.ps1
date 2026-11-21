# Spiri72 updater. Run it with update.cmd (double-click), from Spiri72's folder.
#
# It fetches the newest build from GitHub, checks it against its checksum, and
# replaces spiri72.exe (the old one is kept as spiri72.exe.old). It touches
# nothing of yours: spiri72.ini, the books, the spirits' books and the rites
# stay as they are. Spiri72 itself never goes online; only this does, and only
# when you run it.
#
# While Spiri72 is being tested, it follows the test builds: the release named
# "proba", which every passing build replaces. Later it will follow only
# finished releases (set $Canalis to "latest").

param(
    # For testing the updater itself: a folder holding a release (the zip,
    # SHA256SUMS.txt and VERSIO.txt) to update from, in place of GitHub.
    [string]$Fons = "",
    # The folder to update; by default the folder this script is in.
    [string]$Domus = $PSScriptRoot
)

$ErrorActionPreference = "Stop"
$Canalis = "proba"
$Repo = "KeychainofSolomon/Spiri72"
$Zip = "spiri72-windows.zip"
$Sums = "SHA256SUMS.txt"
# What the update may write into your folder, and nothing else.
$Ours = @("spiri72.exe", "update.ps1", "update.cmd", "README.md")

function Say($text) { Write-Host $text }
function Fail($text, $code) { Write-Host ""; Write-Host "Not updated: $text" -ForegroundColor Red; exit $code }

Say "Spiri72 updater"
Say "Folder: $Domus"

if (Get-Process -Name spiri72 -ErrorAction SilentlyContinue) {
    Fail "Spiri72 is running. Close it, then run the updater again." 3
}

$versioFile = Join-Path $Domus "spiri72.versio"
$current = ""
if (Test-Path $versioFile) { $current = (Get-Content $versioFile -Raw).Trim() }

$work = Join-Path ([IO.Path]::GetTempPath()) ("spiri72-update-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $zipPath = Join-Path $work $Zip
    $sumsPath = Join-Path $work $Sums
    $notes = ""
    if ($Fons -ne "") {
        Copy-Item (Join-Path $Fons $Zip) $zipPath
        Copy-Item (Join-Path $Fons $Sums) $sumsPath
    } else {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $headers = @{ "User-Agent" = "spiri72-updater"; "Accept" = "application/vnd.github+json" }
        if ($Canalis -eq "latest") {
            $url = "https://api.github.com/repos/$Repo/releases/latest"
        } else {
            $url = "https://api.github.com/repos/$Repo/releases/tags/$Canalis"
        }
        try {
            $release = Invoke-RestMethod -Uri $url -Headers $headers
        } catch {
            Fail "GitHub could not be reached, or there is no build yet ($($_.Exception.Message))." 4
        }
        $notes = $release.name
        $assetZip = $release.assets | Where-Object { $_.name -eq $Zip } | Select-Object -First 1
        $assetSums = $release.assets | Where-Object { $_.name -eq $Sums } | Select-Object -First 1
        if (-not $assetZip -or -not $assetSums) { Fail "the newest build has no $Zip or $Sums." 4 }
        Say "Downloading $($release.name)..."
        Invoke-WebRequest -Uri $assetZip.browser_download_url -OutFile $zipPath -UseBasicParsing -Headers $headers
        Invoke-WebRequest -Uri $assetSums.browser_download_url -OutFile $sumsPath -UseBasicParsing -Headers $headers
    }

    # the download must match its checksum, or nothing is changed
    $line = Get-Content $sumsPath | Where-Object { $_ -match ("\s\*?" + [regex]::Escape($Zip) + "$") } | Select-Object -First 1
    if (-not $line) { Fail "$Sums names no $Zip." 5 }
    $expected = ($line -split "\s+")[0].ToLower()
    $actual = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) { Fail "the download does not match its checksum; it may be damaged. Nothing was changed." 5 }

    $unpacked = Join-Path $work "unpacked"
    Expand-Archive -Path $zipPath -DestinationPath $unpacked
    $newExe = Join-Path $unpacked "spiri72.exe"
    if (-not (Test-Path $newExe)) { Fail "the build holds no spiri72.exe." 5 }
    $versioNew = ""
    $versioPath = Join-Path $unpacked "VERSIO.txt"
    if (Test-Path $versioPath) { $versioNew = (Get-Content $versioPath -Raw).Trim() }
    if ($versioNew -ne "" -and $versioNew -eq $current) {
        Say ""
        Say "Already up to date: $current"
        exit 0
    }

    # the old program is kept beside the new, and put back if anything fails
    $exe = Join-Path $Domus "spiri72.exe"
    $old = Join-Path $Domus "spiri72.exe.old"
    if (Test-Path $exe) { Copy-Item $exe $old -Force }
    try {
        foreach ($name in $Ours) {
            $from = Join-Path $unpacked $name
            if (Test-Path $from) { Copy-Item $from (Join-Path $Domus $name) -Force }
        }
    } catch {
        if (Test-Path $old) { Copy-Item $old $exe -Force }
        Fail "the new files could not be written ($($_.Exception.Message)); the old program was put back." 6
    }
    if ($versioNew -ne "") { Set-Content -Path $versioFile -Value $versioNew -Encoding ASCII }

    Say ""
    if ($current -eq "") { $current = "an unknown build" }
    Write-Host "Updated from $current to $versioNew." -ForegroundColor Green
    if ($notes -ne "") { Say $notes }
    Say "The previous program is kept as spiri72.exe.old."
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
