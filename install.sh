#!/usr/bin/env bash
# install.sh — Instala Aspire-Team (y sus dependencias: Rust, Aurelia)
# Uso: curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
#      o: ./install.sh
set -euo pipefail

BOLD="\033[1m"; GREEN="\033[32m"; YELLOW="\033[33m"; RED="\033[31m"; RESET="\033[0m"
info()  { echo -e "${BOLD}${GREEN}▸${RESET} $*"; }
warn()  { echo -e "${BOLD}${YELLOW}!${RESET} $*"; }
error() { echo -e "${BOLD}${RED}✗${RESET} $*" >&2; exit 1; }

# ---------- OS ----------
OS="$(uname -s 2>/dev/null || echo Unknown)"
case "$OS" in
    Linux*)  PLATFORM=linux ;;
    Darwin*) PLATFORM=macos ;;
    *)       error "Sistema no soportado: $OS (usa install.ps1 en Windows)" ;;
esac

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
    if [[ "$PLATFORM" == "linux" ]] && command -v yay >/dev/null 2>&1; then
        yay -S --noconfirm aurelia || cargo install aurelia
    elif [[ "$PLATFORM" == "linux" ]] && command -v pacman >/dev/null 2>&1; then
        warn "Aurelia no está en los repos oficiales de Arch; instala con: yay -S aurelia"
        cargo install aurelia
    elif [[ "$PLATFORM" == "macos" ]]; then
        cargo install aurelia
    else
        cargo install aurelia
    fi
else
    info "Aurelia: $(aurelia --version 2>&1 | head -1)"
fi

# ---------- Compilar Aspire-Team ----------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"
info "Compilando Aspire-Team..."
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

echo ""
info "${BOLD}Listo!${RESET} Ejecuta con: ${BOLD}aspire-team${RESET}"
echo "  Variable opcional: ASPIRE_AURELIA_BIN para usar otro binario de aurelia."