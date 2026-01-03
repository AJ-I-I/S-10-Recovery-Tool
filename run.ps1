# S-10 Recovery Tool - PowerShell Launcher Script
# This script builds and runs the S-10 Recovery Tool

param(
    [string]$Target = "",
    [string]$Pattern = "",
    [string]$Output = "",
    [switch]$Deep = $false,
    [switch]$Build = $false,
    [switch]$Release = $false,
    [switch]$Help = $false
)

# Script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# Display help
if ($Help) {
    Write-Host "S-10 Recovery Tool - PowerShell Launcher" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\run.ps1 [options]"
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Target <path>     Target directory or disk to scan"
    Write-Host "  -Pattern <regex>   Search pattern (regex)"
    Write-Host "  -Output <path>     Output directory for recovered files"
    Write-Host "  -Deep              Enable deep scan (slower but more thorough)"
    Write-Host "  -Build             Build the project before running"
    Write-Host "  -Release           Build in release mode (optimized)"
    Write-Host "  -Help              Show this help message"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\run.ps1 -Target C:\Users\Documents"
    Write-Host "  .\run.ps1 -Target C:\ -Pattern '\.txt$' -Deep"
    Write-Host "  .\run.ps1 -Target D:\ -Output C:\Recovered -Release"
    exit 0
}

# Check if Rust/Cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Cargo (Rust toolchain) is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Build the project if requested or if binary doesn't exist
$BuildMode = if ($Release) { "release" } else { "dev" }
$BinaryPath = if ($Release) { 
    "target\release\s10-recovery-tool.exe" 
} else { 
    "target\debug\s10-recovery-tool.exe" 
}

if ($Build -or -not (Test-Path $BinaryPath)) {
    Write-Host "Building project in $BuildMode mode..." -ForegroundColor Cyan
    
    if ($Release) {
        $buildResult = cargo build --release 2>&1
    } else {
        $buildResult = cargo build 2>&1
    }
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        $buildResult | Write-Host
        exit 1
    }
    
    Write-Host "Build completed successfully!" -ForegroundColor Green
}

# Check if binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Host "Error: Binary not found at $BinaryPath" -ForegroundColor Red
    Write-Host "Try running with -Build flag" -ForegroundColor Yellow
    exit 1
}

# Build command arguments
$argsList = @()

if ($Target) {
    $argsList += "--target"
    $argsList += $Target
}

if ($Pattern) {
    $argsList += "--pattern"
    $argsList += $Pattern
}

if ($Output) {
    $argsList += "--output"
    $argsList += $Output
}

if ($Deep) {
    $argsList += "--deep"
}

# Run the application
Write-Host "Starting S-10 Recovery Tool..." -ForegroundColor Cyan
Write-Host ""

& $BinaryPath $argsList

# Check exit code
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Application exited with error code: $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}

