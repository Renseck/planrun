<#
.SYNOPSIS
Installs planrun on Windows: the latest release, or a build of this clone (D83).

.DESCRIPTION
Without -FromSource: downloads the latest release's x86_64-pc-windows-msvc archive from GitHub
and puts planrun.exe in -InstallDir. With -FromSource: builds this clone with
`cargo install --locked` into the same directory. Either way -InstallDir is added to the user
PATH when it is missing, and `planrun update` keeps the install current.

One line, without a clone:
  irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex

.PARAMETER FromSource
Build the clone this script is in, instead of downloading a release. Needs cargo.

.PARAMETER InstallDir
Where planrun.exe goes. Default: $env:LOCALAPPDATA\Programs\planrun\bin. With -FromSource its
name must be bin (cargo installs into <root>\bin).

.PARAMETER NoPath
Leave the user PATH as it is.
#>
param(
    [switch]$FromSource,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\planrun\bin'),
    [switch]$NoPath
)

$ErrorActionPreference = 'Stop'
$repo = 'Renseck/planrun'
$target = 'x86_64-pc-windows-msvc'

function Install-FromSource {
    param([string]$Dir)
    if (-not $PSScriptRoot) {
        throw 'planrun: -FromSource builds a clone: git clone https://github.com/Renseck/planrun, then run scripts\install.ps1 -FromSource in it'
    }
    $root = Split-Path -Parent $PSScriptRoot
    if (-not (Test-Path (Join-Path $root 'crates\planrun-cli\Cargo.toml'))) {
        throw "planrun: $root is not a planrun clone"
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'planrun: -FromSource needs cargo on PATH (https://rustup.rs)'
    }
    if ((Split-Path -Leaf $Dir) -ne 'bin') {
        throw "planrun: -FromSource needs an -InstallDir named bin (cargo installs into <root>\bin): $Dir"
    }
    $cargoRoot = Split-Path -Parent $Dir
    cargo install --path (Join-Path $root 'crates\planrun-cli') --locked --force --root $cargoRoot
    if ($LASTEXITCODE -ne 0) { throw "planrun: cargo install failed (exit code $LASTEXITCODE)" }
}

function Install-Release {
    param([string]$Dir)
    # Windows PowerShell 5.1 (the one-liner may run there) must be told to use TLS 1.2.
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $headers = @{ 'User-Agent' = 'planrun-install'; 'Accept' = 'application/vnd.github+json' }
    $release = Invoke-RestMethod -Headers $headers "https://api.github.com/repos/$repo/releases/latest"
    $name = "planrun-$($release.tag_name)-$target.zip"
    $asset = $release.assets | Where-Object name -eq $name
    if (-not $asset) { throw "planrun: release $($release.tag_name) has no $name" }
    $temp = Join-Path ([System.IO.Path]::GetTempPath()) "planrun-install-$PID"
    New-Item -ItemType Directory -Force $temp | Out-Null
    try {
        $zip = Join-Path $temp $name
        Invoke-WebRequest -Headers $headers -UseBasicParsing -Uri $asset.browser_download_url -OutFile $zip
        Expand-Archive -Path $zip -DestinationPath $temp -Force
        New-Item -ItemType Directory -Force $Dir | Out-Null
        Copy-Item (Join-Path $temp 'planrun.exe') (Join-Path $Dir 'planrun.exe') -Force
    } finally {
        Remove-Item -Recurse -Force $temp -ErrorAction SilentlyContinue
    }
}

function Add-ToUserPath {
    param([string]$Dir)
    # Read and write the registry value as it is: [Environment]::SetEnvironmentVariable would
    # expand the %VARIABLES% in the user PATH and store it as a plain string.
    $key = Get-Item 'HKCU:\Environment'
    $path = $key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
    $entries = @($path -split ';' | Where-Object { $_ })
    if ($entries -contains $Dir) { return }
    Set-ItemProperty 'HKCU:\Environment' -Name Path -Value (($entries + $Dir) -join ';') -Type ExpandString
    # Setting and clearing a variable through .NET tells running programs that the environment
    # changed, so a terminal started from Explorer sees the new PATH.
    [Environment]::SetEnvironmentVariable('PLANRUN_INSTALL', '1', 'User')
    [Environment]::SetEnvironmentVariable('PLANRUN_INSTALL', $null, 'User')
    $env:Path = "$env:Path;$Dir"
    "added $Dir to the user PATH (new terminals see it)"
}

$InstallDir = [System.IO.Path]::GetFullPath($InstallDir).TrimEnd('\')
if ($FromSource) { Install-FromSource $InstallDir } else { Install-Release $InstallDir }
if (-not $NoPath) { Add-ToUserPath $InstallDir }
& (Join-Path $InstallDir 'planrun.exe') --version
