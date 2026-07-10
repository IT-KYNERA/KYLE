# install.ps1 — Windows PowerShell installer for Kyle
#
# Usage:
#   iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
#
# Environment variables:
#   $env:KY_VERSION = "v0.6.1"     Version to install (default: latest)
#   $env:KY_PREFIX = "C:\ky"       Install directory (default: ~\.ky)

param(
    [string]$Version = "v0.6.2",
    [string]$Prefix = ""
)

$Repo = "IT-KYNERA/KYLE"

# ─── Detect architecture ────────────────────────────────────

$Arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "arm64" }
$Platform = "windows-$Arch"
$Bundle = "ky-windows-$Arch.zip"
$BundleUrl = "https://github.com/$Repo/releases/download/$Version/$Bundle"

Write-Host "Detected: $Platform"

# ─── Determine install prefix ───────────────────────────────

if (-not $Prefix) {
    $env:KY_PREFIX = if ($env:KY_PREFIX) { $env:KY_PREFIX } else { "$env:USERPROFILE\.ky" }
} else {
    $env:KY_PREFIX = $Prefix
}

$BinDir = "$env:KY_PREFIX\bin"
$LibDir = "$env:KY_PREFIX\lib"

# ─── Download ───────────────────────────────────────────────

$TmpDir = "$env:TEMP\ky-install"
if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

$OutFile = "$TmpDir\$Bundle"
Write-Host "Downloading Kyle $Version for $Platform..."
Write-Host "  $BundleUrl"

try {
    Invoke-WebRequest -Uri $BundleUrl -OutFile $OutFile -UseBasicParsing
} catch {
    Write-Host "Error: failed to download $Bundle"
    Write-Host "Check that $Version exists at:"
    Write-Host "  https://github.com/$Repo/releases"
    exit 1
}

# ─── Extract ─────────────────────────────────────────────────

Write-Host "Extracting..."
try {
    Expand-Archive -Path $OutFile -DestinationPath $TmpDir -Force
} catch {
    Write-Host "Error: failed to extract $Bundle"
    exit 1
}

if (-not (Test-Path "$TmpDir\ky.exe")) {
    Write-Host "Error: ky.exe not found in archive"
    Get-ChildItem $TmpDir
    exit 1
}

# ─── Install ─────────────────────────────────────────────────

Write-Host "Installing to $env:KY_PREFIX..."

# Create directories
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $LibDir | Out-Null

# Copy files
Copy-Item "$TmpDir\ky.exe" "$BinDir\ky.exe" -Force
if (Test-Path "$TmpDir\libkyc_runtime.a") {
    Copy-Item "$TmpDir\libkyc_runtime.a" "$LibDir\libkyc_runtime.a" -Force
} elseif (Test-Path "$TmpDir\kyc_runtime.lib") {
    Copy-Item "$TmpDir\kyc_runtime.lib" "$LibDir\kyc_runtime.lib" -Force
}

# Clean up ky install temp
Remove-Item -Recurse -Force $TmpDir

# ─── Install LLVM 18.1.8 (runtime dependency) ──────────────
# ky.exe dynamically links against LLVM-C.dll at runtime.
# We download the NSIS installer and extract it with 7-Zip.
# If 7-Zip is not available, warn the user to install LLVM manually.

$LLVMDir = "$env:KY_PREFIX\llvm-18"
$LLVMExe = "$env:TEMP\LLVM-18.1.8-win64.exe"
$LLVMUrl = "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.exe"

function Find-7z {
    $paths = @(
        "C:\Program Files\7-Zip\7z.exe",
        "C:\Program Files (x86)\7-Zip\7z.exe",
        "$env:ProgramFiles\7-Zip\7z.exe",
        "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
    )
    foreach ($p in $paths) { if (Test-Path $p) { return $p } }
    $cmd = Get-Command "7z" -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

if (-not (Test-Path "$LLVMDir\bin\LLVM-C.dll")) {
    $7zPath = Find-7z
    if (-not $7zPath) {
        Write-Host "Warning: LLVM 18.1.8 is required at runtime by ky.exe."
        Write-Host "Install manually:"
        Write-Host "  1. Download from: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8"
        Write-Host "  2. Run: LLVM-18.1.8-win64.exe"
        Write-Host "  3. Or with Chocolatey: choco install llvm --version=18.1.8"
        Write-Host "Then set:  `$env:LLVM_SYS_181_PREFIX = 'C:\Program Files\LLVM'"
        Write-Host ""
    } else {
        Write-Host "Downloading LLVM 18.1.8 (required at runtime by ky)..."
        try {
            Invoke-WebRequest -Uri $LLVMUrl -OutFile $LLVMExe -UseBasicParsing
        } catch {
            Write-Host "Warning: failed to download LLVM. ky.exe needs LLVM-C.dll at runtime."
            Write-Host "Install manually from: https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8"
            Write-Host ""
        }
        if (Test-Path $LLVMExe) {
            Write-Host "Extracting LLVM 18.1.8 with 7-Zip..."
            New-Item -ItemType Directory -Force -Path $LLVMDir | Out-Null
            & $7zPath x $LLVMExe -o"$LLVMDir" -y | Out-Null
            Remove-Item $LLVMExe -Force
            Write-Host "  LLVM installed to $LLVMDir"
        }
    }
} else {
    Write-Host "LLVM 18.1.8 already installed at $LLVMDir"
}

$LLVMBin = "$LLVMDir\bin"
$env:LLVM_SYS_181_PREFIX = $LLVMDir

# ─── Add to PATH ────────────────────────────────────────────

$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$addedDirs = @()
if ($UserPath -notlike "*$BinDir*") { $addedDirs += $BinDir }
if ($UserPath -notlike "*$LLVMBin*") { $addedDirs += $LLVMBin }
if ($addedDirs.Count -gt 0) {
    $NewPath = ($addedDirs -join ';') + ";" + $UserPath
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    $env:PATH = ($addedDirs -join ';') + ";" + $env:PATH
    Write-Host "  Added to PATH:"
    foreach ($d in $addedDirs) { Write-Host "    $d" }
} else {
    Write-Host "  PATH already configured"
}

# ─── Verify ─────────────────────────────────────────────────

Write-Host ""
try {
    $version = & "$BinDir\ky.exe" --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Kyle $Version installed successfully!"
        Write-Host ""
        Write-Host "  Binary:  $BinDir\ky.exe"
        if (Test-Path "$LibDir\libkyc_runtime.a" -or (Test-Path "$LibDir\kyc_runtime.lib")) {
            Write-Host "  Runtime: installed"
        }
        if (Test-Path "$LLVMDir\bin\LLVM-C.dll") {
            Write-Host "  LLVM:    $LLVMDir"
        } else {
            Write-Host "  LLVM:    NOT INSTALLED — ky.exe needs LLVM-C.dll at runtime"
        }
        Write-Host ""
        Write-Host "  Use now:  ky --version"
        Write-Host "  Try:      ky run examples\hello.ky"
    } else {
        Write-Host "⚠️  Installation completed but verification failed."
    }
} catch {
    Write-Host "⚠️  Installation completed but 'ky.exe' not found in PATH."
    Write-Host "   Restart your terminal or add manually:"
    Write-Host "    [Environment]::SetEnvironmentVariable('PATH', ""$BinDir;`$env:PATH"", 'User')"
}
