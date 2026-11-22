# install.ps1 — Instala Aspire-Team en Windows (y sus dependencias: Rust, Aurelia)
# Uso: iwr -useb https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.ps1 | iex
#      o: .\install.ps1
# Requiere: PowerShell como Administrador
$ErrorActionPreference = "Continue"

function Banner {
    Write-Host ""
    Write-Host "    █████╗ ███████╗██████╗ ██╗██████╗ ███████╗" -ForegroundColor Cyan
    Write-Host "   ██╔══██╗██╔════╝██╔══██╗██║██╔══██╗██╔════╝" -ForegroundColor Cyan
    Write-Host "   ███████║███████╗██████╔╝██║██████╔╝█████╗  " -ForegroundColor Cyan
    Write-Host "   ██╔══██║╚════██║██╔═══╝ ██║██╔══██╗██╔══╝  " -ForegroundColor Cyan
    Write-Host "   ██║  ██║███████║██║     ██║██║  ██║███████╗" -ForegroundColor Cyan
    Write-Host "   ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   TUI retro para Aurelia — cliente de Steam en terminal" -ForegroundColor DarkGray
    Write-Host ""
}

function Info  { Write-Host "  ▸ $args" -ForegroundColor Green }
function Warn  { Write-Host "  ! $args" -ForegroundColor Yellow }
function Fail  { Write-Host "  ✗ $args" -ForegroundColor Red }
function Step  { Write-Host ""; Write-Host "▸ $args" -ForegroundColor Blue }

Banner

# ---------- Verificar admin ----------
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Warn "Se requieren permisos de Administrador."
    Warn "Ejecuta PowerShell como Administrador y vuelve a correr el script."
    Fail "Sin permisos de administrador."
    exit 1
}
Info "Permisos de administrador OK"

# ---------- Rust ----------
Step "Verificando Rust..."
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Info "Rust no encontrado. Instalando..."
    $rustInstalled = $false
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        try {
            winget install --id Rustlang.Rustup -e --silent --accept-package-agreements --accept-source-agreements 2>$null
            $rustInstalled = $true
        } catch {
            Warn "winget falló, intentando rustup directo..."
        }
    }
    if (-not $rustInstalled -or -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        try {
            $rustupUrl = "https://win.rustup.rs/x86_64"
            $rustupPath = "$env:TEMP\rustup-init.exe"
            Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -ErrorAction Stop
            & $rustupPath -y
            $rustInstalled = $true
        } catch {
            Fail "No se pudo instalar Rust. Instálalo manualmente desde https://rustup.rs/"
            exit 1
        }
    }
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
}
if (Get-Command rustc -ErrorAction SilentlyContinue) {
    Info "Rust: $(rustc --version)"
} else {
    Fail "Rust no se instaló correctamente."
    exit 1
}

# ---------- Aurelia ----------
Step "Instalando Aurelia..."
if (-not (Get-Command aurelia -ErrorAction SilentlyContinue)) {
    $aureliaInstalled = $false
    $arch = if ([Environment]::Is64BitOperatingSystem) {
        if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x86_64" }
    } else { "x86_64" }

    # Intentar binario precompilado
    try {
        $releaseUrl = "https://github.com/Drackrath/Aurelia/releases/latest/download/aurelia_windows_$arch.exe"
        $aureliaPath = "$env:USERPROFILE\.cargo\bin\aurelia.exe"
        if (-not (Test-Path "$env:USERPROFILE\.cargo\bin")) {
            New-Item -ItemType Directory -Path "$env:USERPROFILE\.cargo\bin" -Force | Out-Null
        }
        Invoke-WebRequest -Uri $releaseUrl -OutFile $aureliaPath -ErrorAction Stop
        Info "Aurelia descargado desde GitHub Releases ($arch)."
        $aureliaInstalled = $true
    } catch {
        Warn "No se pudo descargar el binario. Intentando cargo install..."
    }

    # Fallback: cargo install
    if (-not $aureliaInstalled) {
        try {
            cargo install aurelia-steam
            $aureliaInstalled = $true
        } catch {
            try {
                cargo install aurelia
                $aureliaInstalled = $true
            } catch {
                Fail "No se pudo instalar Aurelia."
                exit 1
            }
        }
    }
} else {
    Info "Aurelia: $(aurelia --version 2>&1 | Select-Object -First 1)"
}

# ---------- Compilar Aspire-Team ----------
Step "Compilando Aspire-Team..."
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $ScriptDir) { $ScriptDir = Get-Location }
Set-Location $ScriptDir
Info "Esto puede tardar unos minutos..."

$buildOk = $false
try {
    cargo build --release
    if ($LASTEXITCODE -eq 0) { $buildOk = $true }
} catch {
    Warn "Primer intento falló, reintentando..."
}
if (-not $buildOk) {
    try {
        cargo build --release
        if ($LASTEXITCODE -eq 0) { $buildOk = $true }
    } catch {
        Fail "No se pudo compilar Aspire-Team."
        exit 1
    }
}
try {
    cargo install --path . --force
} catch {
    Warn "cargo install falló, pero el binario está en target/release/"
}
Info "Aspire-Team compilado e instalado"

# ---------- Config ----------
Step "Configurando..."
$ConfigDir = Join-Path $env:APPDATA "aspire-team"
if (-not (Test-Path $ConfigDir)) { New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null }
$ConfigFile = Join-Path $ConfigDir "config.toml"
if (-not (Test-Path $ConfigFile)) {
    Set-Content -Path $ConfigFile -Value 'theme = "Sistema"'
    Info "Config creada en $ConfigFile"
} else {
    Info "Config existente conservada"
}
$LogoFile = Join-Path $ConfigDir "logo.txt"
$LogoSrc = Join-Path $ScriptDir "assets\logo.txt"
if (-not (Test-Path $LogoFile) -and (Test-Path $LogoSrc)) {
    Copy-Item -Path $LogoSrc -Destination $LogoFile
    Info "Logo copiado a $LogoFile"
}

Write-Host ""
Write-Host "╔══════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║     ¡Instalación completada con éxito!   ║" -ForegroundColor Green
Write-Host "╚══════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Info "Listo! Ejecuta con: aspire-team"
Write-Host "  • Variable opcional: ASPIRE_AURELIA_BIN para usar otro binario de aurelia."
Write-Host "  • Si ya tienes Steam con sesión iniciada, Aurelia la reutiliza automáticamente."