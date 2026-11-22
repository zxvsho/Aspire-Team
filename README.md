# Aspire-Team

TUI retro (ratatui) para [Aurelia](https://github.com/Drackrath/Aurelia): biblioteca, instalaciones, tienda, mods y parches, complementos, social y sistema.

Backend: Aurelia (cliente de Steam por línea de comandos en Rust). Aspire-Team llama a `aurelia --json <comando>` como subproceso y muestra el resultado.

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/c56170b1-48d5-4483-a7b5-5fc98fa2b1b4" />


## Requisitos

- [Rust](https://rustup.rs/) (1.97 o superior)
- [Aurelia](https://github.com/Drackrath/Aurelia) instalado (se instala automáticamente con el script)

Aurelia gestiona la sesión de Steam por su cuenta: si ya tienes Steam con sesión iniciada, la reutiliza; si no, Aurelia te permite iniciar sesión dentro de la app. No necesitas tener Steam instalado para usar Aspire-Team.

## Instalación rápida (un comando)

El script pide la contraseña de sudo (en Linux) e instala todo automáticamente: dependencias del sistema, Rust, Aurelia y Aspire-Team. Funciona en cualquier distro Linux.

### Linux (cualquier distro: Arch, Debian, Fedora, openSUSE, Alpine, etc.)

```bash
curl -fsSL https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.sh | bash
```

El script pide la contraseña de sudo e instala todo automáticamente: dependencias del sistema, Rust, Aurelia y Aspire-Team. Funciona en cualquier distro Linux. Los comandos son tolerantes a fallos: si un método falla, intenta el siguiente.

### Windows (PowerShell como Administrador)

```powershell
iwr -useb https://raw.githubusercontent.com/zxvsho/Aspire-Team/main/install.ps1 | iex
```

Requiere PowerShell ejecutado como Administrador. El script instala Rust (winget o rustup), Aurelia (winget o cargo) y compila Aspire-Team. Si un paso falla, intenta el siguiente método automáticamente.

### Clonando el repo

```bash
git clone https://github.com/zxvsho/Aspire-Team.git
cd Aspire-Team
chmod +x install.sh && ./install.sh   # Linux
# Windows: .\install.ps1 (como Administrador)
```

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

Aurelia gestiona juegos de Steam. Si ya tienes Steam con sesión iniciada, Aurelia la reutiliza automáticamente; si no, puedes iniciar sesión desde la app (Sistema → Iniciar sesión). No necesitas tener Steam instalado para usar Aspire-Team.

Códigos de salida propios: 80-84 (login QR), 85 `disk_full`, 86 `offline`, 87 `steam_config_mismatch`.

## Plugins y complementos

Aspire-Team expone todo lo que ofrece Aurelia para instalar y gestionar complementos:

| Plugin | Comandos en Aspire-Team |
|---|---|
| **Proton** (GE, CachyOS, etc.) | Complementos → Proton: listar / instalar / predeterminado / desinstalar |
| **umu-launcher** | Complementos → umu: estado / activar / instalar / desinstalar |
| **luxtorpeda** (motor nativo) | Complementos → luxtorpeda: estado / activar / instalar / desinstalar |
| **Steam Runtime** (Wine) | Complementos → Steam runtime: instalar / iniciar sesión / reparar |
| **Scripts de lanzamiento** | Mods y parches → Scripts: crear / ver / eliminar |
| **Parámetros de lanzamiento** | Biblioteca → `o` (opciones) o `config game --launch-options` |
| **Parches y mods locales** | Biblioteca → Enter → "Aplicar parche/mod local" (copia carpeta con respaldo) |
| **Workshop** | Mods y parches → Workshop: explorar / instalar / suscribirse |
| **Depots / versiones** | Mods y parches → Depots, Ramas, Degradar, Fijar versión |

Funciona en Linux.

- **Aurelia** — https://github.com/Drackrath/Aurelia (cliente de Steam por línea de comandos en Rust). Aspire-Team es un frontend que llama a `aurelia --json`.
- **ratatui** — https://github.com/ratatui/ratatui (framework TUI en Rust).
- **LazyVim** — inspiración para la barra lateral, línea de estado y tecla líder.
