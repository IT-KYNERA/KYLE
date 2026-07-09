param(
    [string]$LLVM_PREFIX = "",
    [string]$LLVM_URL = "https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/LLVM-18.1.8-win64.zip"
)

# Find LLVM prefix
if (-not $LLVM_PREFIX) {
    $LLVM_PREFIX = $env:LLVM_SYS_181_PREFIX
}
if (-not $LLVM_PREFIX) {
    # Default: assume script lives in repo tools/windows/ and LLVM is at repo root/llvm-18
    $LLVM_PREFIX = Join-Path (Split-Path (Split-Path $PSScriptRoot -Parent) -Parent) "llvm-18"
}
if (-not (Test-Path $LLVM_PREFIX)) {
    Write-Host "LLVM prefix not found at: $LLVM_PREFIX"
    Write-Host "Downloading LLVM 18.1.8 for Windows..."
    $tmpZip = "$env:TEMP\llvm-18.zip"
    Invoke-WebRequest -Uri $LLVM_URL -OutFile $tmpZip -UseBasicParsing
    New-Item -ItemType Directory -Force -Path $LLVM_PREFIX | Out-Null
    $extractDir = "$env:TEMP\llvm-extract"
    if (Test-Path $extractDir) { Remove-Item -Recurse -Force $extractDir }
    Expand-Archive -Path $tmpZip -DestinationPath $extractDir
    # The zip contains a top-level LLVM-18.1.8-win64/ dir
    Get-ChildItem "$extractDir\LLVM-18.1.8-win64" | Move-Item -Destination $LLVM_PREFIX -Force
    Remove-Item -Recurse $extractDir -Force -ErrorAction SilentlyContinue
    Remove-Item $tmpZip -Force
    Write-Host "LLVM 18.1.8 installed to $LLVM_PREFIX"
} else {
    Write-Host "LLVM prefix found at: $LLVM_PREFIX"
}

# ─── Create llvm-config.h ────────────────────────────────────────
$configDir = Join-Path $LLVM_PREFIX "include\llvm\Config"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

$configH = Join-Path $configDir "llvm-config.h"
Write-Host "Creating $configH..."
Set-Content -Path $configH -Value @'
#ifndef LLVM_CONFIG_H
#define LLVM_CONFIG_H

/* LLVM version */
#define LLVM_VERSION_MAJOR 18
#define LLVM_VERSION_MINOR 1
#define LLVM_VERSION_PATCH 8
#define LLVM_VERSION_STRING "18.1.8"

/* Package info */
#define LLVM_PACKAGE_NAME "LLVM"
#define LLVM_PACKAGE_VERSION "18.1.8"
#define LLVM_PACKAGE_BUGREPORT "https://bugs.llvm.org/"

/* Target triple */
#define LLVM_DEFAULT_TARGET_TRIPLE "x86_64-pc-windows-msvc"
#define LLVM_HOST_TRIPLE "x86_64-pc-windows-msvc"

/* Features */
#define LLVM_ENABLE_ASSERTIONS 0
#define LLVM_ENABLE_EXCEPTIONS 1
#define LLVM_ENABLE_RTTI 0
#define LLVM_ENABLE_THREADS 1
#define LLVM_ENABLE_ZLIB 0
#define LLVM_ENABLE_ZSTD 0
#define LLVM_ENABLE_LIBXML2 0
#define LLVM_ENABLE_TERMINFO 0
#define LLVM_ENABLE_FFI 0
#define LLVM_ENABLE_PLUGINS 1
#define LLVM_ENABLE_EH 1
#define LLVM_ENABLE_CRASH_OVERRIDES 1
#define LLVM_ENABLE_BACKTRACES 1
#define LLVM_ENABLE_DIA_SDK 0

/* Includes */
#define LLVM_INCLUDE_TESTS 0
#define LLVM_INCLUDE_DOCS 0
#define LLVM_INCLUDE_EXAMPLES 0
#define LLVM_INCLUDE_BENCHMARKS 0
#define LLVM_INCLUDE_GO_TESTS 0

/* Native target — X86 for Windows */
#define LLVM_NATIVE_ARCH X86
#define LLVM_NATIVE_TARGET LLVMInitializeX86Target
#define LLVM_NATIVE_TARGETINFO LLVMInitializeX86TargetInfo
#define LLVM_NATIVE_TARGETMC LLVMInitializeX86TargetMC
#define LLVM_NATIVE_ASM LLVMInitializeX86AsmPrinter
#define LLVM_NATIVE_ASMPRINTER LLVMInitializeX86AsmPrinter
#define LLVM_NATIVE_ASMPARSER LLVMInitializeX86AsmParser
#define LLVM_NATIVE_DISASSEMBLER LLVMInitializeX86Disassembler

/* Build mode */
#define LLVM_BUILD_MODE "Release"

/* Endian */
#define LLVM_LITTLE_ENDIAN 1
#define LLVM_BIG_ENDIAN 0

#endif
'@ -Encoding ASCII

# ─── Create .def files ──────────────────────────────────────────
$defs = @{
    "Targets.def"        = "X86`r`nAArch64`r`nARM"
    "AsmPrinters.def"    = "X86`r`nAArch64`r`nARM"
    "AsmParsers.def"     = "X86`r`nAArch64`r`nARM"
    "Disassemblers.def"  = "X86`r`nAArch64`r`nARM"
}
foreach ($name in $defs.Keys) {
    $path = Join-Path $configDir $name
    Write-Host "Creating $path..."
    Set-Content -Path $path -Value $defs[$name] -Encoding ASCII
}

# ─── Create llvm-config.cmd ──────────────────────────────────────
$binDir = Join-Path $LLVM_PREFIX "bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$llvmConfigCmd = Join-Path $binDir "llvm-config.cmd"
Write-Host "Creating $llvmConfigCmd..."
Set-Content -Path $llvmConfigCmd -Value '@echo off
setlocal EnableDelayedExpansion
set "SCRIPT_DIR=%~dp0"
for %%I in ("%SCRIPT_DIR%..") do set "PREFIX=%%~fI"
set "PREFIX_FWD=%PREFIX:\=/%"
if /i "%~1"==""      echo 18.1.8 & goto :eof
if /i "%~1"=="--version"       echo 18.1.8 & goto :eof
if /i "%~1"=="--prefix"        echo %PREFIX% & goto :eof
if /i "%~1"=="--includedir"    echo %PREFIX%\include & goto :eof
if /i "%~1"=="--libdir"        echo %PREFIX%\lib & goto :eof
if /i "%~1"=="--bindir"        echo %PREFIX%\bin & goto :eof
if /i "%~1"=="--cflags"        echo -I%PREFIX_FWD%/include & goto :eof
if /i "%~1"=="--cxxflags"      echo -I%PREFIX_FWD%/include & goto :eof
if /i "%~1"=="--ldflags"       echo -LIBPATH:%PREFIX_FWD%/lib & goto :eof
if /i "%~1"=="--libs"          echo LLVM-C.lib & goto :eof
if /i "%~1"=="--libnames"      echo LLVM-C.lib & goto :eof
if /i "%~1"=="--libfiles"      echo %PREFIX_FWD%/lib/LLVM-C.lib & goto :eof
if /i "%~1"=="--components"    echo all & goto :eof
if /i "%~1"=="--shared-mode"   echo shared & goto :eof
if /i "%~1"=="--system-libs"   echo( & goto :eof
if /i "%~1"=="--targets-built" echo AArch64 ARM X86 & goto :eof
if /i "%~1"=="--host-target"   echo x86_64-pc-windows-msvc & goto :eof
if /i "%~1"=="--has-rtti"      echo NO & goto :eof
if /i "%~1"=="--assertion-mode" echo OFF & goto :eof
if /i "%~1"=="--build-mode"    echo Release & goto :eof
if /i "%~1"=="--link-shared"   echo -DLLVM_LINK_SHARED=1 & goto :eof
if /i "%~1"=="--link-static"   echo( & goto :eof
if /i "%~1"=="--obj-root"      echo %PREFIX_FWD% & goto :eof
if /i "%~1"=="--src-root"      echo %PREFIX_FWD% & goto :eof
exit /b 1
'@ -Encoding ASCII

# ─── Compile llvm-config.exe (if C compiler available) ─────────
$cSrcDir = Join-Path $LLVM_PREFIX "src"
New-Item -ItemType Directory -Force -Path $cSrcDir | Out-Null
$cSrc = Join-Path $cSrcDir "llvm-config.c"

Set-Content -Path $cSrc -Value @'
#include <stdio.h>
#include <string.h>
#include <windows.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char path[MAX_PATH];
    GetModuleFileNameA(NULL, path, MAX_PATH);
    char *p = strrchr(path, '\\');
    if (p) *p = '\0';
    char prefix[MAX_PATH];
    strcpy(prefix, path);
    p = strrchr(prefix, '\\');
    if (p && _stricmp(p + 1, "bin") == 0) *p = '\0';
    char fwd[MAX_PATH];
    for (int i = 0; prefix[i]; i++) fwd[i] = (prefix[i] == '\\') ? '/' : prefix[i];
    fwd[strlen(prefix)] = '\0';
    const char *a = argv[1];
    if (strcmp(a, "--version")==0)       { puts("18.1.8"); return 0; }
    if (strcmp(a, "--prefix")==0)        { puts(prefix); return 0; }
    if (strcmp(a, "--includedir")==0)    { printf("%s\\include\n", prefix); return 0; }
    if (strcmp(a, "--libdir")==0)        { printf("%s\\lib\n", prefix); return 0; }
    if (strcmp(a, "--bindir")==0)        { printf("%s\\bin\n", prefix); return 0; }
    if (strcmp(a, "--cflags")==0)        { printf("-I%s/include\n", fwd); return 0; }
    if (strcmp(a, "--cxxflags")==0)      { printf("-I%s/include\n", fwd); return 0; }
    if (strcmp(a, "--ldflags")==0)       { printf("-LIBPATH:%s/lib\n", fwd); return 0; }
    if (strcmp(a, "--libs")==0)          { puts("LLVM-C.lib"); return 0; }
    if (strcmp(a, "--libnames")==0)      { puts("LLVM-C.lib"); return 0; }
    if (strcmp(a, "--libfiles")==0)      { printf("%s/lib/LLVM-C.lib\n", fwd); return 0; }
    if (strcmp(a, "--components")==0)    { puts("all"); return 0; }
    if (strcmp(a, "--shared-mode")==0)   { puts("shared"); return 0; }
    if (strcmp(a, "--system-libs")==0)   { putchar(10); return 0; }
    if (strcmp(a, "--targets-built")==0) { puts("AArch64 ARM X86"); return 0; }
    if (strcmp(a, "--host-target")==0)   { puts("x86_64-pc-windows-msvc"); return 0; }
    if (strcmp(a, "--has-rtti")==0)      { puts("NO"); return 0; }
    if (strcmp(a, "--assertion-mode")==0){ puts("OFF"); return 0; }
    if (strcmp(a, "--build-mode")==0)    { puts("Release"); return 0; }
    if (strcmp(a, "--link-shared")==0)   { puts("-DLLVM_LINK_SHARED=1"); return 0; }
    if (strcmp(a, "--link-static")==0)   { putchar(10); return 0; }
    if (strcmp(a, "--obj-root")==0)      { puts(fwd); return 0; }
    if (strcmp(a, "--src-root")==0)      { puts(fwd); return 0; }
    return 1;
}
'@ -Encoding ASCII

# Try to compile with cl.exe (MSVC)
$clPath = Get-Command "cl.exe" -ErrorAction SilentlyContinue
if ($clPath) {
    Write-Host "Compiling llvm-config.exe..."
    $clExe = Join-Path $binDir "llvm-config.exe"
    $obj = "$env:TEMP\llvm-config.obj"
    & $clPath.Source "/nologo" "/Fo:$obj" "/Fe:$clExe" $cSrc 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0 -and (Test-Path $clExe)) {
        Write-Host "  → $clExe ($((Get-Item $clExe).Length / 1024) KB)"
    } else {
        Write-Host "  → cl.exe compile failed, using llvm-config.cmd fallback"
    }
} else {
    Write-Host "  → no C compiler found, using llvm-config.cmd"
}

Write-Host ""
Write-Host "LLVM setup complete."
Write-Host "  Prefix: $LLVM_PREFIX"
