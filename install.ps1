# install.ps1 — Instala Aspire-Team en Windows (y sus dependencias: Rust, Aurelia)
# Uso: iwr -useb https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.ps1 | iex
#      o: .\install.ps1
$ErrorActionPreference = "Stop"

function Info  { Write-Host "▸ $args" -ForegroundColor Green }
function Warn  { Write-Host "! $args" -ForegroundColor Yellow }
function Fail  { Write-Host "✗ $args" -ForegroundColor Red; exit 1 }

# ---------- Rust ----------
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Info "Instalando Rust via rustup..."
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Rustlang.Rustup -e --silent
    } else {
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        & $rustupPath -y
    }
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}
Info "Rust: $(rustc --version)"

# ---------- Aurelia ----------
if (-not (Get-Command aurelia -ErrorAction SilentlyContinue)) {
    Info "Instalando Aurelia..."
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Drackrath.Aurelia -e --silent 2>$null
        if (-not (Get-Command aurelia -ErrorAction SilentlyContinue)) {
            cargo install aurelia
        }
    } else {
        cargo install aurelia
    }
} else {
    Info "Aurelia: $(aurelia --version 2>&1 | Select-Object -First 1)"
}

# ---------- Compilar Aspire-Team ----------
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir
Info "Compilando Aspire-Team..."
cargo build --release
cargo install --path . --force

# ---------- Config ----------
$ConfigDir = Join-Path $env:APPDATA "aspire-team"
if (-not (Test-Path $ConfigDir)) { New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null }
$ConfigFile = Join-Path $ConfigDir "config.toml"
if (-not (Test-Path $ConfigFile)) {
    Set-Content -Path $ConfigFile -Value 'theme = "Sistema"'
    Info "Config creada en $ConfigFile"
}
# Copy logo if not present
$LogoFile = Join-Path $ConfigDir "logo.txt"
$LogoSrc = Join-Path $ScriptDir "assets\logo.txt"
if (-not (Test-Path $LogoFile) -and (Test-Path $LogoSrc)) {
    Copy-Item -Path $LogoSrc -Destination $LogoFile
    Info "Logo copiado a $LogoFile"
}

Write-Host ""
Info "Listo! Ejecuta con: aspire-team"
Write-Host "  Variable opcional: ASPIRE_AURELIA_BIN para usar otro binario de aurelia."