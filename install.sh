#!/usr/bin/env bash
# install.sh — Instala Aspire-Team y sus dependencias (Rust, Aurelia)
# Funciona en cualquier distro Linux.
# Uso: curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
#      o: ./install.sh
set -euo pipefail

# Asegurar directorio de trabajo válido
cd "$HOME" 2>/dev/null || cd /tmp 2>/dev/null || true

# ---------- Colores y estilos ----------
BOLD="\033[1m"
DIM="\033[2m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
CYAN="\033[36m"
MAGENTA="\033[35m"
BLUE="\033[34m"
WHITE="\033[37m"
RESET="\033[0m"

# ---------- Funciones de UI ----------
banner() {
    echo ""
    echo -e "${CYAN}${BOLD}"
    echo "    █████╗ ███████╗██████╗ ██╗██████╗ ███████╗"
    echo "   ██╔══██╗██╔════╝██╔══██╗██║██╔══██╗██╔════╝"
    echo "   ███████║███████╗██████╔╝██║██████╔╝█████╗  "
    echo "   ██╔══██║╚════██║██╔═══╝ ██║██╔══██╗██╔══╝  "
    echo "   ██║  ██║███████║██║     ██║██║  ██║███████╗"
    echo "   ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝"
    echo -e "${RESET}"
    echo -e "${DIM}   TUI retro para Aurelia — cliente de Steam en terminal${RESET}"
    echo ""
}

step() { echo -e "\n${BOLD}${BLUE}▸ $*${RESET}"; }
ok()   { echo -e "  ${GREEN}✔${RESET} $*"; }
warn() { echo -e "  ${YELLOW}!${RESET} $*"; }
fail() { echo -e "  ${RED}✗${RESET} $*" >&2; }
info() { echo -e "  ${DIM}$*${RESET}"; }

progress_bar() {
    local current=$1 total=$2 width=40
    local filled=$((current * width / total))
    local empty=$((width - filled))
    local bar=""
    for ((i=0; i<filled; i++)); do bar+="█"; done
    for ((i=0; i<empty; i++)); do bar+="░"; done
    echo -ne "\r  ${CYAN}[${bar}]${RESET} ${current}/${total}"
}

# ---------- OS ----------
OS="$(uname -s 2>/dev/null || echo Unknown)"
case "$OS" in
    Linux*)  PLATFORM=linux ;;
    *)       fail "Sistema no soportado: $OS"; exit 1 ;;
esac

banner
step "Detectando plataforma..."
ok "Linux detectado"

# ---------- sudo ----------
if [[ $EUID -ne 0 ]]; then
    step "Permisos de administrador"
    if ! command -v sudo >/dev/null 2>&1; then
        fail "sudo no está instalado. Instálalo primero."
        exit 1
    fi
    info "Se necesita sudo para instalar dependencias."
    info "Ingresa tu contraseña cuando se solicite:"
    if ! sudo -v; then
        fail "No se pudo validar sudo. Verifica tu contraseña."
        exit 1
    fi
    ok "sudo validado"
fi

run_sudo() {
    if [[ $EUID -ne 0 ]]; then
        sudo "$@"
    else
        "$@"
    fi
}

# ---------- Rust ----------
step "Verificando Rust..."
if ! command -v cargo >/dev/null 2>&1; then
    info "Rust no encontrado. Instalando via rustup..."
    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        fail "Se necesita curl o wget para instalar Rust"
        exit 1
    fi
    source "$HOME/.cargo/env" 2>/dev/null || true
    export PATH="$HOME/.cargo/bin:$PATH"
fi
ok "Rust: $(rustc --version)"

# ---------- Dependencias del sistema ----------
step "Instalando dependencias del sistema..."
DEPS_OK=false
if command -v apt-get >/dev/null 2>&1; then
    run_sudo apt-get update -qq 2>/dev/null && run_sudo apt-get install -y -qq build-essential pkg-config libssl-dev 2>/dev/null && DEPS_OK=true
elif command -v pacman >/dev/null 2>&1; then
    run_sudo pacman -S --needed --noconfirm base-devel openssl 2>/dev/null && DEPS_OK=true
elif command -v dnf >/dev/null 2>&1; then
    run_sudo dnf install -y gcc openssl-devel 2>/dev/null && DEPS_OK=true
elif command -v zypper >/dev/null 2>&1; then
    run_sudo zypper install -y gcc openssl-devel 2>/dev/null && DEPS_OK=true
elif command -v apk >/dev/null 2>&1; then
    run_sudo apk add --no-cache build-base openssl-dev 2>/dev/null && DEPS_OK=true
elif command -v xbps-install >/dev/null 2>&1; then
    run_sudo xbps-install -y base-devel openssl 2>/dev/null && DEPS_OK=true
else
    warn "Gestor de paquetes no reconocido. Asegúrate de tener gcc y openssl."
fi
if [[ "$DEPS_OK" == "true" ]]; then ok "Dependencias instaladas"; else warn "Dependencias parciales, continuando..."; fi

# ---------- Aurelia ----------
step "Instalando Aurelia..."
if ! command -v aurelia >/dev/null 2>&1; then
    INSTALLED=false
    if command -v pacman >/dev/null 2>&1; then
        if run_sudo pacman -S --needed --noconfirm aurelia 2>/dev/null; then INSTALLED=true; fi
    elif command -v apt-get >/dev/null 2>&1; then
        if run_sudo apt-get install -y -qq aurelia 2>/dev/null; then INSTALLED=true; fi
    elif command -v dnf >/dev/null 2>&1; then
        if run_sudo dnf install -y aurelia 2>/dev/null; then INSTALLED=true; fi
    elif command -v zypper >/dev/null 2>&1; then
        if run_sudo zypper install -y aurelia 2>/dev/null; then INSTALLED=true; fi
    fi
    if [[ "$INSTALLED" != "true" ]]; then
        warn "Aurelia no está en repos oficiales. Instalando via cargo..."
        cargo install aurelia
    fi
    ok "Aurelia instalado"
else
    ok "Aurelia: $(aurelia --version 2>&1 | head -1)"
fi

# ---------- Compilar Aspire-Team ----------
step "Compilando Aspire-Team..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
info "Esto puede tardar unos minutos..."
BUILD_OK=false
if cargo build --release 2>&1 | tail -3; then
    BUILD_OK=true
else
    warn "Primer intento falló, reintentando..."
    if cargo build --release 2>&1 | tail -3; then
        BUILD_OK=true
    fi
fi
if [[ "$BUILD_OK" != "true" ]]; then
    fail "No se pudo compilar Aspire-Team."
    exit 1
fi
cargo install --path . --force 2>&1 | tail -1 || warn "cargo install falló, pero el binario está en target/release/"
ok "Aspire-Team compilado e instalado"

# ---------- Config ----------
step "Configurando..."
mkdir -p "$HOME/.config/aspire-team"
if [[ ! -f "$HOME/.config/aspire-team/config.toml" ]]; then
    cat > "$HOME/.config/aspire-team/config.toml" <<'EOF'
theme = "Sistema"
EOF
    ok "Config creada en ~/.config/aspire-team/config.toml"
else
    ok "Config existente conservada"
fi
if [[ ! -f "$HOME/.config/aspire-team/logo.txt" ]] && [[ -f "$SCRIPT_DIR/assets/logo.txt" ]]; then
    cp "$SCRIPT_DIR/assets/logo.txt" "$HOME/.config/aspire-team/logo.txt"
    ok "Logo copiado a ~/.config/aspire-team/logo.txt"
fi

# ---------- Final ----------
echo ""
echo -e "${GREEN}${BOLD}╔══════════════════════════════════════════╗${RESET}"
echo -e "${GREEN}${BOLD}║     ¡Instalación completada con éxito!   ║${RESET}"
echo -e "${GREEN}${BOLD}╚══════════════════════════════════════════╝${RESET}"
echo ""
echo -e "  Ejecuta con: ${BOLD}aspire-team${RESET}"
echo -e "  ${DIM}Variable opcional: ASPIRE_AURELIA_BIN para otro binario de aurelia.${RESET}"
echo -e "  ${DIM}Si ya tienes Steam con sesión iniciada, Aurelia la reutiliza.${RESET}"
echo ""