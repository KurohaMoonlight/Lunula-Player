Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Building Lunula Audio Player Installer  " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$isccPath = "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe"
if (-not (Test-Path $isccPath)) {
    $isccCmd = Get-Command iscc.exe -ErrorAction SilentlyContinue
    if ($isccCmd) {
        $isccPath = $isccCmd.Source
    } else {
        Write-Error "Inno Setup compiler (ISCC.exe) not found!"
        exit 1
    }
}

Write-Host "Step 1: Compiling Rust Release Binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed!"
    exit 1
}

Write-Host "Step 2: Creating Installer with Inno Setup..." -ForegroundColor Yellow
if (-not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" | Out-Null
}

& "$isccPath" "installer.iss"
if ($LASTEXITCODE -eq 0) {
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host " Installer Successfully Created in dist/! " -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Get-Item "dist\LunulaAudioPlayer_Setup_*.exe" | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
} else {
    Write-Error "Installer compilation failed!"
}

