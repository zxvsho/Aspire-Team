# Aspire-Team

TUI retro (ratatui) para [Aurelia](https://github.com/Drackrath/Aurelia): biblioteca, instalaciones, tienda, mods y parches, complementos, social y sistema.

Backend: Aurelia (cliente de Steam por línea de comandos en Rust). Aspire-Team llama a `aurelia --json <comando>` como subproceso y muestra el resultado.

## Requisitos

- [Rust](https://rustup.rs/) (1.97 o superior)
- [Aurelia](https://github.com/Drackrath/Aurelia) instalado y con sesión iniciada (`aurelia login`)
- Steam con una cuenta con sesión iniciada (Aurelia la reutiliza automáticamente)

## Instalación rápida (un comando)

### Linux (Arch / CachyOS / Artix)

```bash
curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
```

O clonando el repo:

```bash
git clone https://github.com/zxvsho/Aspire-Team.git
cd Aspire-Team
chmod +x install.sh && ./install.sh
```

### Windows (PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.ps1 | iex
```

O clonando el repo:

```powershell
git clone https://github.com/zxvsho/Aspire-Team.git
cd Aspire-Team
.\install.ps1
```

### macOS

```bash
curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
```

El script instala Rust (si no está), Aurelia (si no está) y compila Aspire-Team.

## Instalación manual

```bash
# 1. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Instalar Aurelia
# Arch: yay -S aurelia
# Otros: cargo install aurelia

# 3. Compilar Aspire-Team
git clone https://github.com/zxvsho/Aspire-Team.git
cd Aspire-Team
cargo install --path .

# 4. Ejecutar
aspire-team
```

## Uso

```bash
aspire-team
```

Variable `ASPIRE_AURELIA_BIN` para usar otro binario de aurelia.

## Controles

### Teclado (vim)

| Tecla | Acción |
|---|---|
| `j` `k` | barra lateral: navegar secciones · contenido: mover |
| `Enter` | seleccionar sección / ejecutar acción |
| `l` | entrar al contenido |
| `h` | volver a la barra lateral |
| `gg` `G` | inicio / fin de la barra o lista |
| `gt` `gT` | sección siguiente / anterior |
| `g1`..`g8` | ir a la sección N |
| `0` `$` | inicio / fin de la lista |
| `Ctrl-d` `Ctrl-u` | media página |
| `Ctrl-f` `Ctrl-b` | página completa |
| `H` `M` `L` | arriba / centro / abajo |
| `/` `n` `N` | filtrar · siguiente · anterior |
| `Space` | tecla líder (which-key) |
| `:` | comando libre de aurelia |
| `T` | cambiar tema |
| `v` | alternar resultado formateado / crudo |
| `[` `]` `J` | cambiar de trabajo · panel de trabajos |
| `?` | ayuda |
| `q` `Ctrl-c` | salir |

### Ratón

- **Rueda** arriba/abajo: navegar.
- **Clic izquierdo** en la barra lateral: cambiar de sección.
- **Clic izquierdo** en la lista: seleccionar juego/acción.
- **Clic derecho**: cerrar overlay/modales.

### Biblioteca

| Tecla | Acción |
|---|---|
| `/` | filtrar |
| `i` | alternar solo instalados |
| `p` | jugar |
| `o` | opciones de lanzamiento |
| `I` | instalar |
| `u` | actualizar |
| `K` | información |
| `a` | menú completo del juego |

## Temas

`T` abre el selector. Incluye 11 temas:

```
Retro82 · Terminal · Sistema · Ambar · Synthwave84 · Commodore64
Gruvbox · Nord · Dracula · TokyoNight · Catppuccin Mocha
```

- **Terminal**: usa la paleta ANSI de tu terminal.
- **Sistema**: lee `~/.config/theme-engine/colors.css` (o `colors.rasi`) y se recarga cuando `color-engine.sh` cambia los colores.

Se guarda en `~/.config/aspire-team/config.toml`.

## Logo

Pon tu logo en ASCII/texto plano en `~/.config/aspire-team/logo.txt` (se muestra en Inicio).

## Parches locales

Biblioteca → Enter → «Aplicar parche/mod local»: copia una carpeta sobre el juego con respaldo en `.aspire-backup/`; «Revertir» lo deshace.
`aurelia verify` restaura los archivos originales.

## Acerca de Aurelia

Aurelia gestiona juegos de Steam de la cuenta con sesión iniciada. Si ya tienes Steam con sesión iniciada, Aurelia la reutiliza automáticamente (no necesitas volver a iniciar sesión en Steam). Solo ejecuta `aurelia login` la primera vez.

Códigos de salida propios: 80-84 (login QR), 85 `disk_full`, 86 `offline`, 87 `steam_config_mismatch`.

## Licencia

GPL-3.0-or-later