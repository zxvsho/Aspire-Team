#!/usr/bin/env bash
# install.sh — Instala Aspire-Team y sus dependencias (Rust, Aurelia)
# Funciona en cualquier distro Linux.
# Uso: curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
#      o: ./install.sh
set -euo pipefail

# Asegurar directorio de trabajo válido (evita getcwd errors)
cd "$HOME" 2>/dev/null || cd /tmp 2>/dev/null || true

BOLD="\033[1m"; GREEN="\033[32m"; YELLOW="\033[33m"; RED="\033[31m"; CYAN="\033[36m"; RESET="\033[0m"
info()  { echo -e "${BOLD}${GREEN}▸${RESET} $*"; }
warn()  { echo -e "${BOLD}${YELLOW}!${RESET} $*"; }
error() { echo -e "${BOLD}${RED}✗${RESET} $*" >&2; exit 1; }
ask()   { echo -en "${BOLD}${CYAN}?${RESET} $* "; }

# ---------- OS ----------
OS="$(uname -s 2>/dev/null || echo Unknown)"
case "$OS" in
    Linux*)  PLATFORM=linux ;;
    *)       error "Sistema no soportado: $OS. En Windows usa install.ps1" ;;
esac

info "Plataforma detectada: Linux"

# ---------- sudo password ----------
SUDO=""
if [[ "$PLATFORM" == "linux" ]]; then
    if [[ $EUID -ne 0 ]]; then
        ask "Se necesita sudo para instalar dependencias. Contraseña:"
        # Leer desde /dev/tty para que funcione con curl | bash
        if [[ -t 0 ]]; then
            read -rs SUDO_PASS
        else
            read -rs SUDO_PASS < /dev/tty
        fi
        echo ""
        if [[ -z "$SUDO_PASS" ]]; then
            error "No se ingresó contraseña"
        fi
        if ! echo "$SUDO_PASS" | sudo -S true 2>/dev/null; then
            error "Contraseña incorrecta"
        fi
        SUDO="echo $SUDO_PASS | sudo -S"
    fi
fi

run_sudo() {
    if [[ -n "$SUDO" ]]; then
        eval "$SUDO $*"
    else
        sudo "$@"
    fi
}

# ---------- Rust ----------
if ! command -v cargo >/dev/null 2>&1; then
    info "Instalando Rust via rustup..."
    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        error "Se necesita curl o wget para instalar Rust"
    fi
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env" 2>/dev/null || true
    export PATH="$HOME/.cargo/bin:$PATH"
fi
info "Rust: $(rustc --version)"

# ---------- Dependencias del sistema ----------
info "Instalando dependencias del sistema..."
if command -v apt-get >/dev/null 2>&1; then
    run_sudo apt-get update -qq
    run_sudo apt-get install -y -qq build-essential pkg-config libssl-dev 2>/dev/null || true
elif command -v pacman >/dev/null 2>&1; then
    run_sudo pacman -S --needed --noconfirm base-devel openssl 2>/dev/null || true
elif command -v dnf >/dev/null 2>&1; then
    run_sudo dnf install -y gcc openssl-devel 2>/dev/null || true
elif command -v zypper >/dev/null 2>&1; then
    run_sudo zypper install -y gcc openssl-devel 2>/dev/null || true
elif command -v apk >/dev/null 2>&1; then
    run_sudo apk add --no-cache build-base openssl-dev 2>/dev/null || true
elif command -v xbps-install >/dev/null 2>&1; then
    run_sudo xbps-install -y base-devel openssl 2>/dev/null || true
else
    warn "Gestor de paquetes no reconocido. Asegúrate de tener gcc y openssl instalados."
fi

# ---------- Aurelia ----------
if ! command -v aurelia >/dev/null 2>&1; then
    info "Instalando Aurelia..."
    INSTALLED=false
    # Intentar repos oficiales según distro
    if command -v pacman >/dev/null 2>&1; then
        if run_sudo pacman -S --needed --noconfirm aurelia 2>/dev/null; then
            INSTALLED=true
        fi
    elif command -v apt-get >/dev/null 2>&1; then
        if run_sudo apt-get install -y -qq aurelia 2>/dev/null; then
            INSTALLED=true
        fi
    elif command -v dnf >/dev/null 2>&1; then
        if run_sudo dnf install -y aurelia 2>/dev/null; then
            INSTALLED=true
        fi
    elif command -v zypper >/dev/null 2>&1; then
        if run_sudo zypper install -y aurelia 2>/dev/null; then
            INSTALLED=true
        fi
    fi
    # Fallback: cargo install (funciona en cualquier distro)
    if [[ "$INSTALLED" != "true" ]]; then
        warn "Aurelia no está en los repos oficiales. Instalando via cargo..."
        cargo install aurelia
    fi
else
    info "Aurelia: $(aurelia --version 2>&1 | head -1)"
fi

# ---------- Compilar Aspire-Team ----------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
info "Compilando Aspire-Team (esto puede tardar unos minutos)..."
BUILD_OK=false
if cargo build --release 2>&1; then
    BUILD_OK=true
else
    warn "Primer intento falló, reintentando..."
    if cargo build --release 2>&1; then
        BUILD_OK=true
    fi
fi
if [[ "$BUILD_OK" != "true" ]]; then
    error "No se pudo compilar Aspire-Team. Verifica que Rust esté instalado correctamente."
fi
cargo install --path . --force || warn "cargo install falló, pero el binario está en target/release/"

# ---------- Config ----------
mkdir -p "$HOME/.config/aspire-team"
if [[ ! -f "$HOME/.config/aspire-team/config.toml" ]]; then
    cat > "$HOME/.config/aspire-team/config.toml" <<'EOF'
theme = "Sistema"
EOF
    info "Config creada en ~/.config/aspire-team/config.toml"
fi
if [[ ! -f "$HOME/.config/aspire-team/logo.txt" ]] && [[ -f "$SCRIPT_DIR/assets/logo.txt" ]]; then
    cp "$SCRIPT_DIR/assets/logo.txt" "$HOME/.config/aspire-team/logo.txt"
    info "Logo copiado a ~/.config/aspire-team/logo.txt"
fi

echo ""
info "${BOLD}¡Listo!${RESET} Ejecuta con: ${BOLD}aspire-team${RESET}"
echo "  • Variable opcional: ASPIRE_AURELIA_BIN para usar otro binario de aurelia."
echo "  • Si ya tienes Steam con sesión iniciada, Aurelia la reutiliza automáticamente."