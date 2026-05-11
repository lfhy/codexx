param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\CodexX')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$repo = $env:CODEXX_RELEASE_REPO
$tag = $env:CODEXX_RELEASE_TAG
$version = $env:CODEXX_RELEASE_VERSION
$arch = $env:CODEXX_RELEASE_ARCH
$commit = $env:CODEXX_RELEASE_COMMIT

foreach ($entry in @(
    @{ Name = 'CODEXX_RELEASE_REPO'; Value = $repo },
    @{ Name = 'CODEXX_RELEASE_TAG'; Value = $tag },
    @{ Name = 'CODEXX_RELEASE_VERSION'; Value = $version },
    @{ Name = 'CODEXX_RELEASE_ARCH'; Value = $arch },
    @{ Name = 'CODEXX_RELEASE_COMMIT'; Value = $commit }
)) {
    if ([string]::IsNullOrWhiteSpace($entry.Value)) {
        throw "$($entry.Name) is required"
    }
}

$assetName = "codexx-windows-$arch.exe"
$assetUrl = "https://github.com/$repo/releases/download/$tag/$assetName"
$tempFile = Join-Path $env:TEMP $assetName

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Invoke-WebRequest -Uri $assetUrl -OutFile $tempFile
Copy-Item $tempFile (Join-Path $InstallDir 'codexx.exe') -Force

$shortCommit = $commit.Substring(0, 7)
@"
name=CodexX
version=$version
commit=$commit
short_commit=$shortCommit
profile=release
platform=windows
arch=$arch
artifact=codexx.exe
built_at=$([DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ"))
"@ | Set-Content -Path (Join-Path $InstallDir 'codexx.version.txt') -Encoding utf8

$currentUserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ([string]::IsNullOrWhiteSpace($currentUserPath)) {
    [Environment]::SetEnvironmentVariable('Path', $InstallDir, 'User')
} elseif (-not ($currentUserPath.Split(';') -contains $InstallDir)) {
    [Environment]::SetEnvironmentVariable('Path', "$InstallDir;$currentUserPath", 'User')
}

Write-Host "Installed CodexX to $InstallDir"
