# install-extension.ps1 — Windows PowerShell installer for Kyle VS Code extension
#
# Usage:
#   iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.ps1" | iex
#
# Environment variables:
#   $env:KY_EXT_VERSION = "v0.7.0"   Version tag to download VSIX from

param(
    [string]$Version = "v0.7.0"
)

$Repo = "IT-KYNERA/KYLE"
$TmpDir = "$env:TEMP\ky-ext-$(Get-Random)"

Write-Host "==> Kyle VS Code Extension — Installer (Windows)"
Write-Host ""

# --- Find VS Code CLI ---
$codePath = Get-Command "code" -ErrorAction SilentlyContinue
if (-not $codePath) {
    $codePaths = @(
        "$env:LOCALAPPDATA\Programs\Microsoft VS Code\bin\code.cmd",
        "$env:ProgramFiles\Microsoft VS Code\bin\code.cmd",
        "${env:ProgramFiles(x86)}\Microsoft VS Code\bin\code.cmd"
    )
    foreach ($p in $codePaths) {
        if (Test-Path $p) { $codePath = $p; break }
    }
}
if (-not $codePath) {
    Write-Host "ERROR: VS Code 'code' CLI not found."
    Write-Host "Open VS Code -> Ctrl+Shift+P -> 'Shell Command: Install 'code' command in PATH'"
    exit 1
}
if ($codePath -is [System.Management.Automation.CommandInfo]) {
    $codeExe = $codePath.Source
} else {
    $codeExe = $codePath
}
Write-Host "VS Code found: $codeExe"

# --- Uninstall mode ---
if ($args[0] -eq "uninstall") {
    Write-Host "Removing Kyle VS Code extension..."
    & $codeExe --uninstall-extension kynera.ky 2>$null
    Write-Host "Extension uninstalled."
    exit 0
}

# --- Download VSIX ---
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
$vsixFile = "$TmpDir\ky-extension.vsix"
$vsixUrl = "https://github.com/$Repo/releases/download/$Version/ky-extension.vsix"

Write-Host "Downloading extension from $vsixUrl..."
try {
    Invoke-WebRequest -Uri $vsixUrl -OutFile $vsixFile -UseBasicParsing
} catch {
    Write-Host "ERROR: failed to download VSIX from release."
    Write-Host "Build it manually: cd vscode-ky && npm install && npx @vscode/vsce package"
    Remove-Item -Recurse $TmpDir -Force -ErrorAction SilentlyContinue
    exit 1
}

if (-not (Test-Path $vsixFile)) {
    Write-Host "ERROR: VSIX download failed."
    Remove-Item -Recurse $TmpDir -Force -ErrorAction SilentlyContinue
    exit 1
}

# --- Install ---
Write-Host "Installing extension..."
& $codeExe --install-extension $vsixFile --force 2>&1 | Out-Null

Remove-Item -Recurse $TmpDir -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "[OK] Kyle VS Code extension installed!"
Write-Host ""
Write-Host "Verify:  code --list-extensions | Select-String ky"
Write-Host ""
Write-Host "Uninstall: iwr -Uri 'https://raw.githubusercontent.com/$Repo/main/vscode-ky/install-extension.ps1' | iex -args uninstall"
