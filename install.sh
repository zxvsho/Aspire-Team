#!/usr/bin/env bash
# install.sh — Instala Aspire-Team y sus dependencias (Rust, Aurelia)
# Uso: curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
#      o: ./install.sh
set -euo pipefail

BOLD="\033[1m"; GREEN="\033[32m"; YELLOW="\033[33m"; RED="\033[31m"; CYAN="\033[36m"; RESET="\033[0m"
info()  { echo -e "${BOLD}${GREEN}▸${RESET} $*"; }
warn()  { echo -e "${BOLD}${YELLOW}!${RESET} $*"; }
error() { echo -e "${BOLD}${RED}✗${RESET} $*" >&2; exit 1; }
ask()   { echo -en "${BOLD}${CYAN}?${RESET} $* "; }

# ---------- OS ----------
OS="$(uname -s 2>/dev/null || echo Unknown)"
case "$OS" in
    Linux*)  PLATFORM=linux ;;
    Darwin*) PLATFORM=macos ;;
    *)       error "Sistema no soportado: $OS. En Windows usa install.ps1" ;;
esac

info "Plataforma detectada: $PLATFORM"

# ---------- sudo password ----------
SUDO=""
if [[ "$PLATFORM" == "linux" ]]; then
    if [[ $EUID -ne 0 ]]; then
        ask "Se necesita sudo para instalar dependencias. Contraseña:"
        read -rs SUDO_PASS
        echo ""
        # verify
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
    else
        error "Se necesita curl para instalar Rust"
    fi
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi
info "Rust: $(rustc --version)"

# ---------- Aurelia ----------
if ! command -v aurelia >/dev/null 2>&1; then
    info "Instalando Aurelia..."
    if [[ "$PLATFORM" == "linux" ]]; then
        if command -v yay >/dev/null 2>&1; then
            info "Usando yay (Arch)..."
            yay -S --noconfirm aurelia 2>/dev/null || cargo install aurelia
        elif command -v pacman >/dev/null 2>&1; then
            warn "Aurelia no está en repos oficiales. Instalando via cargo..."
            cargo install aurelia
        elif command -v apt-get >/dev/null 2>&1; then
            warn "Aurelia no está en apt. Instalando via cargo..."
            cargo install aurelia
        else
            cargo install aurelia
        fi
    elif [[ "$PLATFORM" == "macos" ]]; then
        if command -v brew >/dev/null 2>&1; then
            brew install aurelia 2>/dev/null || cargo install aurelia
        else
            cargo install aurelia
        fi
    fi
else
    info "Aurelia: $(aurelia --version 2>&1 | head -1)"
fi

# ---------- Dependencias del sistema (Linux) ----------
if [[ "$PLATFORM" == "linux" ]]; then
    info "Instalando dependencias del sistema..."
    if command -v apt-get >/dev/null 2>&1; then
        run_sudo apt-get update -qq
        run_sudo apt-get install -y -qq build-essential pkg-config libssl-dev 2>/dev/null || true
    elif command -v pacman >/dev/null 2>&1; then
        run_sudo pacman -S --needed --noconfirm base-devel openssl 2>/dev/null || true
    elif command -v dnf >/dev/null 2>&1; then
        run_sudo dnf install -y gcc openssl-devel 2>/dev/null || true
    fi
fi

# ---------- Compilar Aspire-Team ----------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
info "Compilando Aspire-Team (esto puede tardar unos minutos)..."
cargo build --release
cargo install --path . --force

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