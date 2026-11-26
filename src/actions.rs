//! Catálogo de acciones. Plantillas de argumentos:
//! {app} = juego seleccionado · {} = todo lo escrito · {..} = palabras sueltas
//! {1} {2} {3} = n-ésima palabra · {2..} = desde la 2.ª palabra, como un solo argumento

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Job,
    Interactive,
    Launch,
    Patch,
    Revert,
    Pick,
    OpenDir,
    Browse,
    Saves,
    OpenPrefix,
    OpenUrl,
    DiskInfo,
    LaunchPrefix,
    Shell,
    ShellInteractive,
    ExtStatus,
}

pub struct Act {
    pub label: &'static str,
    pub args: &'static str,
    pub ask: Option<&'static str>,
    pub game: bool,
    pub mode: Mode,
    pub tip: &'static str,
}

const fn a(label: &'static str, args: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: None, game: false, mode: Mode::Job, tip }
}
const fn q(label: &'static str, args: &'static str, ask: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: Some(ask), game: false, mode: Mode::Job, tip }
}
const fn p(label: &'static str, args: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: None, game: false, mode: Mode::Pick, tip }
}
const fn g(label: &'static str, args: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: None, game: true, mode: Mode::Job, tip }
}
const fn gq(label: &'static str, args: &'static str, ask: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: Some(ask), game: true, mode: Mode::Job, tip }
}
const fn i(label: &'static str, args: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: None, game: false, mode: Mode::Interactive, tip }
}
const fn iq(label: &'static str, args: &'static str, ask: &'static str, tip: &'static str) -> Act {
    Act { label, args, ask: Some(ask), game: false, mode: Mode::Interactive, tip }
}
const fn m(label: &'static str, mode: Mode, ask: Option<&'static str>, tip: &'static str) -> Act {
    Act { label, args: "", ask, game: true, mode, tip }
}
/// Acción sobre el juego seleccionado sin argumentos.
const fn gm(label: &'static str, mode: Mode, tip: &'static str) -> Act {
    Act { label, args: "", ask: None, game: true, mode, tip }
}
/// Prefijo de opciones de lanzamiento (`mangohud`, `gamemoderun`, …).
const fn gp(label: &'static str, prefix: &'static str, tip: &'static str) -> Act {
    Act { label, args: prefix, ask: None, game: true, mode: Mode::LaunchPrefix, tip }
}
/// Comando de shell con la terminal completa (sudo, instaladores).
const fn shi(label: &'static str, cmd: &'static str, tip: &'static str) -> Act {
    Act { label, args: cmd, ask: None, game: false, mode: Mode::ShellInteractive, tip }
}

/// Menú de un juego (Enter o `a` en Biblioteca).
pub const GAME_ACTS: &[Act] = &[
    g("Jugar", "play {app}", "Lanza el juego con la configuración guardada"),
    m("Jugar con opciones…", Mode::Launch, None, "Plataforma, Proton, umu, scripts, parámetros de lanzamiento"),
    g("Instalar", "install {app}", "Descarga e instala el juego"),
    g("Actualizar", "update {app}", "Descarga el último manifiesto"),
    g("Verificar archivos", "verify {app}", "Comprueba la integridad de los archivos"),
    g("Desinstalar", "uninstall {app}", "Elimina el juego"),
    g("Detener", "stop {app}", "Detiene el juego en ejecución"),
    g("Detener (forzar)", "stop {app} --force", "Forza el stop (SIGKILL) si el juego cuelga"),
    g("Parámetros de lanzamiento", "config game {app}", "Ver/fijar --platform, Proton, launch-options…"),
    g("Información", "info {app}", "Descripción, reseñas, etiquetas, DLC"),
    g("Información extendida", "info {app} --extended", "Requisitos, Metacritic, géneros"),
    g("DLC", "dlc {app}", "DLC con propiedad e instalación"),
    g("Logros", "achievements {app}", "Estado y rareza de tus logros"),
    g("Noticias", "news {app}", "Anuncios recientes"),
    g("Reseñas", "reviews {app}", "Reseñas de usuarios"),
    g("Similares", "similar {app}", "Juegos parecidos según Steam"),
    g("Jugadores ahora", "players {app}", "Jugadores conectados"),
    g("Precio por región", "price {app} --compare US,DE,JP", "Compara precio y descuento"),
    g("Ramas beta", "branches {app}", "Lista las ramas"),
    gq("Cambiar rama", "set-branch {app} {}", "Rama", "Cambia de rama (beta…)"),
    gq("Cambiar idioma", "set-language {app} {}", "Idioma (p. ej. spanish)", "Cambia los depots de idioma"),
    g("Depots", "depots {app}", "Lista los depots"),
    g("Opciones de lanzamiento (Steam)", "launch-options {app}", "Configuraciones de lanzamiento de Steam"),
    g("Destinos de lanzamiento", "launch-targets {app}", "Builds instaladas y flags de play"),
    g("Manifiestos por rama", "manifests {app}", "ID de manifiesto por depot"),
    gq("Degradar versión (parche)", "downgrade {app} --depot {1} --manifest {2}", "Depot Manifiesto", "Instala una build anterior y la fija"),
    g("Fijar versión", "pin {app}", "Bloquea actualizaciones"),
    g("Soltar versión", "unpin {app}", "Permite actualizar de nuevo"),
    gm("Abrir carpeta de instalación", Mode::OpenDir, "Abre la carpeta del juego en el gestor de archivos"),
    gm("Explorar archivos del juego", Mode::Browse, "Navega, abre o borra archivos del juego (F en Biblioteca)"),
    gm("Tamaño y disco", Mode::DiskInfo, "Cuánto ocupa, en qué disco y cuánto espacio queda (D)"),
    gm("Abrir carpeta de guardados", Mode::Saves, "Abre los guardados sincronizados por Steam"),
    gm("Abrir prefijo de Wine", Mode::OpenPrefix, "Carpeta del prefijo (compatdata) del juego"),
    gm("Página del juego en Steam", Mode::OpenUrl, "Abre la ficha del juego en el navegador"),
    m("Parche en español o mod (carpeta)", Mode::Patch, Some("Carpeta del parche o mod"), "Copia el parche sobre el juego con respaldo en .aspire-backup"),
    m("Restaurar archivos originales", Mode::Revert, None, "Restaura lo que copió el último parche local"),
    g("Workshop: explorar", "workshop browse {app}", "Descubre mods del Workshop"),
    g("Workshop: estado", "workshop status {app}", "Instalados vs suscritos"),
    gq("Workshop: instalar item", "workshop install {}", "ID del item", "Descarga un item o colección"),
    gq("Workshop: suscribirse + instalar", "workshop subscribe {} --install", "ID del item", "Suscribe y descarga"),
    gq("Proton del juego", "config game {app} --proton {}", "Versión de Proton (p. ej. GE-Proton9-20)", "Fija un Proton para este juego"),
    gq("Parámetros de lanzamiento", "config game {app} --launch-options {}", "Opciones (p. ej. gamemoderun %command% -novid)", "Sintaxis de Steam con %command%"),
    gq("Prefijo de Wine propio", "config game {app} --prefix {}", "Ruta del prefijo", "Usa un prefijo propio"),
    g("Usar umu en este juego", "config game {app} --umu", "Proton vía umu-run"),
    g("Usar motor nativo (luxtorpeda)", "config game {app} --native-engine", "Motor nativo de Linux"),
    g("Script de lanzamiento: crear", "scripts new {app}", "Crea <script_dir>/<appid>.sh"),
    g("Script de lanzamiento: ver", "scripts show {app}", "Muestra el script resuelto"),
    g("Nube: sincronizar", "cloud sync {app}", "Sincroniza partidas guardadas"),
    g("Nube: archivos", "cloud list {app}", "Lista archivos en Steam Cloud"),
    gq("Mover a otra biblioteca", "move {app} {}", "Carpeta de biblioteca destino", "Mueve la instalación"),
    g("¿Instalado en disco?", "available {app}", "Comprueba presencia en disco"),
];

const INSTALLS: &[Act] = &[
    a("Cola de instalaciones", "install list", "Instalaciones en curso y pendientes"),
    q("Instalar por AppID", "install {}", "AppID", "Descarga e instala"),
    q("Instalar en biblioteca", "install {1} --library {2..}", "AppID carpeta-biblioteca", "Instala en una biblioteca concreta (ej: /home/user/SteamLibrary)"),
    q("Pausar", "install pause {}", "AppID", "Pausa y conserva archivos parciales"),
    q("Reanudar", "install resume {}", "AppID", "Reanuda una instalación"),
    a("Reanudar todas", "install resume --all", "Una por una"),
    q("Detener (cancelar)", "install stop {}", "AppID", "Cancela y deja en la cola"),
    q("Olvidar entrada", "install forget {}", "AppID", "Quita de la cola sin borrar"),
    q("Olvidar y borrar parciales", "install forget {} --delete-files", "AppID", "Quita de la cola y borra parciales"),
    q("Actualizar juego", "update {}", "AppID", "Descarga el último manifiesto"),
    a("Buscar actualizaciones", "list --installed --check-updates", "Estado update/mismatch de instalados"),
    q("Verificar", "verify {}", "AppID", "Integridad de archivos"),
    q("Desinstalar", "uninstall {}", "AppID", "Elimina el juego"),
    a("Bibliotecas (carpetas)", "libraries", "Carpetas de Steam y espacio libre"),
    a("Duplicados", "duplicates", "Juegos instalados en varias bibliotecas"),
    a("En ejecución", "running", "Juegos que Aurelia está ejecutando"),
    q("Detener juego", "stop {}", "AppID", "Detiene un juego en ejecución"),
    q("Importar archivos existentes", "import {1} {2..}", "AppID carpeta-biblioteca", "Registra archivos ya en disco"),
    q("Revincular tras mover", "relink {1} {2..}", "AppID carpeta-biblioteca", "Re-apunta Steam a archivos movidos"),
];

const STORE: &[Act] = &[
    q("Buscar juego", "search {}", "Título (muestra AppIDs)", "Busca AppIDs por título"),
    a("Ofertas", "deals", "Más vendidos con descuento"),
    a("Eventos y rebajas", "events", "Eventos de la tienda"),
    q("Información", "info {}", "AppID de Steam (número)", "Detalles del juego"),
    q("Información extendida", "info {} --extended", "AppID de Steam (número)", "Requisitos, géneros…"),
    q("Precio por región", "price {} --compare US,DE,JP", "AppID de Steam (número)", "Compara regiones"),
    q("Similares", "similar {}", "AppID de Steam (número)", "Juegos parecidos"),
    q("Jugadores ahora", "players {}", "AppID de Steam (número)", "Jugadores conectados"),
    q("Noticias", "news {}", "AppID de Steam (número)", "Anuncios recientes"),
    q("Reseñas", "reviews {}", "AppID de Steam (número)", "Reseñas de usuarios"),
    q("Reseñas negativas", "reviews {} -t negative", "AppID de Steam (número)", "Solo negativas"),
    q("DLC", "dlc {}", "AppID de Steam (número)", "DLC y propiedad"),
    q("Logros", "achievements {}", "AppID de Steam (número)", "Tus logros"),
    q("Verificar DRM/propiedad", "drm {}", "AppID de Steam (número)", "Tickets sin cliente Steam"),
    q("Portada (caché)", "image {}", "AppID de Steam (número)", "Descarga arte y muestra la ruta"),
    q("Imágenes y trailers", "image {} --list --probe", "AppID de Steam (número)", "URLs de arte, capturas y trailers"),
];

const MODS: &[Act] = &[
    q("Workshop: explorar", "workshop browse {}", "AppID", "Descubre mods"),
    q("Workshop: info de item", "workshop info {}", "ID del item", "Metadatos de item o colección"),
    q("Workshop: instalar item", "workshop install {1}", "ID del item", "Descarga un item"),
    q("Workshop: suscribir + instalar", "workshop subscribe {1} --install", "ID del item", "Suscribe y descarga"),
    q("Workshop: estado", "workshop status {}", "AppID", "Instalados vs suscritos"),
    q("Workshop: valorar", "workshop rate {1} {2}", "ID-item up|down", "Pulgar arriba o abajo"),
    q("Workshop: comentarios", "workshop comments {}", "ID del item", "Lee comentarios"),
    q("Workshop: comentar", "workshop comment {1} {2..}", "ID-item texto", "Publica un comentario"),
    m("Parche en español o mod (carpeta)", Mode::Patch, Some("Carpeta del parche o mod"), "Requiere juego elegido en Biblioteca; copia con respaldo"),
    m("Restaurar archivos originales", Mode::Revert, None, "Requiere juego elegido en Biblioteca"),
    q("Manifiestos (versiones)", "manifests {}", "AppID", "Manifiestos por depot y rama"),
    q("Degradar a versión", "downgrade {1} --depot {2} --manifest {3}", "AppID depot manifiesto", "Manifiestos antiguos: steamdb.info"),
    q("Fijar versión", "pin {}", "AppID", "Bloquea actualizaciones"),
    q("Soltar versión", "unpin {}", "AppID", "Permite actualizar"),
    q("Depots", "depots {}", "AppID", "Lista depots"),
    q("Archivos de un depot", "depots files {1} {2}", "AppID depot", "Explora el árbol del manifiesto"),
    q("Descargar archivo de depot", "depots download {1} {2} {3} -o out", "AppID depot ruta/archivo", "Baja un archivo (SHA-1 verificado) a ./out"),
    q("Ramas beta", "branches {}", "AppID", "Lista ramas"),
    q("Cambiar de rama", "set-branch {1} {2}", "AppID rama", "Cambia de rama"),
    q("Scripts: crear", "scripts new {}", "AppID", "Envuelve el lanzamiento con tu script"),
    a("Scripts: listar", "scripts list", "AppIDs con script"),
    q("Scripts: ver", "scripts show {}", "AppID", "Script resuelto y contenido"),
    q("Scripts: eliminar", "scripts remove {}", "AppID", "Borra el script"),
    q("Opciones de lanzamiento (Steam)", "launch-options {}", "AppID", "Lista las configuraciones"),
    q("Parámetros de lanzamiento", "config game {1} --launch-options {2..}", "AppID parámetros…", "Ej: 391540 gamemoderun %command%"),
];

const PLUGINS: &[Act] = &[
    a("Proton: listar", "proton list", "Runtimes instalables e instalados"),
    p("Proton: instalar", "proton install {}", "Elige el runtime de la lista (j/k, Enter)"),
    p("Proton: predeterminado", "proton default {}", "Elige el runtime global por defecto (j/k, Enter)"),
    p("Proton: desinstalar", "proton uninstall {}", "Elige el runtime a borrar (j/k, Enter)"),
    a("Proton: mapeos de Steam", "proton mappings", "Herramientas de compatibilidad"),
    a("Runtimes detectados", "config protons", "Proton y Wine detectados"),
    a("umu: estado", "umu status", "Estado y versión"),
    a("umu: activar", "umu enable", "Proton vía umu-run"),
    a("umu: instalar", "umu install", "Descarga umu-run"),
    q("umu: usar instalación existente", "umu path {}", "Ruta", "Omite la descarga"),
    a("umu: desinstalar", "umu uninstall", "Elimina lo descargado"),
    a("luxtorpeda: estado", "luxtorpeda status", "Motores nativos"),
    a("luxtorpeda: activar", "luxtorpeda enable", "Activa el complemento"),
    a("luxtorpeda: instalar", "luxtorpeda install", "Descarga el cliente"),
    q("luxtorpeda: usar instalación existente", "luxtorpeda path {}", "Ruta", "Omite la descarga"),
    a("luxtorpeda: desinstalar", "luxtorpeda uninstall", "Elimina lo descargado"),
    a("Steam runtime (Wine): estado", "steam-runtime status", "Prefijo maestro de Steam en Wine"),
    p("Steam runtime: elegir runner", "config steam-runtime-runner {}", "Elige el runner de Steam (j/k, Enter)"),
    i("Steam runtime: instalar", "steam-runtime install", "Instala Steam en Wine (inicias sesión ahí)"),
    i("Steam runtime: iniciar sesión", "steam-runtime login", "Abre Steam en Wine"),
    i("Steam runtime: reparar", "steam-runtime repair", "Respalda y reinstala"),
    a("Steam runtime: detener", "steam-runtime stop", "Cierra Steam en Wine"),
    a("Steam runtime: desinstalar", "steam-runtime uninstall", "Borra el prefijo"),
    q("Steam runtime: política", "config steam-runtime-policy {}", "auto | on | off", "Cuándo usar el Steam en Wine"),
    q("Shader precache", "config shader-precache {}", "auto | off", "Precompilar shaders Vulkan"),
    q("Precompilar shaders de un juego", "shaders {}", "AppID", "Linux: ahora mismo"),
    // ---- Extensiones del sistema (MangoHud, Goverlay, vkBasalt, GameMode…) ----
    Act {
        label: "Extensiones: estado",
        args: "",
        ask: None,
        game: false,
        mode: Mode::ExtStatus,
        tip: "Comprueba MangoHud, Goverlay, vkBasalt, GameMode, Gamescope y ProtonUp-Qt",
    },
    shi(
        "Extensiones: instalar paquete",
        crate::tools::EXT_INSTALL,
        "Instala MangoHud, Goverlay, vkBasalt, GameMode y Gamescope con sudo",
    ),
    shi(
        "Extensiones: abrir Goverlay",
        crate::tools::EXT_GOVERLAY,
        "Interfaz gráfica para configurar MangoHud y vkBasalt",
    ),
    shi(
        "ProtonUp-Qt: instalar",
        crate::tools::EXT_PROTONUP,
        "Descarga Proton-GE y Wine-GE fácilmente (Flatpak o paquete)",
    ),
    shi(
        "ProtonUp-Qt / ProtonPlus: abrir",
        crate::tools::EXT_PROTONMANAGER,
        "Abre el gestor de versiones de Proton",
    ),
    gp("MangoHud: activar en este juego", "mangohud", "Añade `mangohud %command%` a las opciones de lanzamiento"),
    gp("MangoHud: desactivar en este juego", "!mangohud", "Quita MangoHud de las opciones de lanzamiento"),
    gp("GameMode: activar en este juego", "gamemoderun", "Añade `gamemoderun %command%` (requiere GameMode)"),
    gp("vkBasalt: activar en este juego", "ENABLE_VKBASALT=1", "Activa vkBasalt con una variable de entorno"),
];

const SOCIAL: &[Act] = &[
    a("Amigos", "friends", "Nombre, estado y juego actual"),
    q("Buscar usuario", "friends search {}", "SteamID64, URL, nombre o me", "Perfil de un usuario"),
    q("Agregar amigo", "friends add {}", "SteamID64 / URL / nombre", "Envía solicitud"),
    q("Quitar amigo", "friends remove {}", "SteamID64 / URL / nombre", "Quita o cancela solicitud"),
    a("Mi perfil", "user me", "Tu perfil según Steam"),
    a("Lista de deseos", "wishlist", "Con precios"),
    q("Lista de deseos de un amigo", "wishlist {}", "Nombre del amigo", "Ver lista de otro"),
    q("Deseos: agregar", "wishlist add {}", "AppID", "Agrega a tu lista"),
    q("Deseos: quitar", "wishlist remove {}", "AppID", "Quita de tu lista"),
    q("Chat: historial", "chat history {}", "SteamID64", "Mensajes recientes"),
    q("Chat: enviar", "chat send {1} {2..}", "SteamID64 mensaje", "Mensaje directo"),
    iq("Chat: sesión en vivo", "chat open {}", "SteamID64", "Chat interactivo (Ctrl-D sale)"),
    q("Presencia", "config presence {}", "online | invisible", "Cómo apareces en el chat"),
    q("Inventario", "inventory {1} --context {2}", "AppID contexto (753 6 = cartas)", "Tu inventario"),
    q("Mercado: precio", "market price {1} {2..}", "AppID nombre-del-item", "Precio de un item"),
    q("Mercado: buscar", "market search {}", "Texto", "Busca en el Mercado"),
    a("Mercado: mis anuncios", "market listings", "Anuncios y órdenes"),
    a("Cartera", "wallet", "Saldo de la cartera"),
    a("Colecciones: listar", "collections list", "Colecciones y conteo"),
    q("Colecciones: crear", "collections create {}", "Nombre", "Nueva colección"),
    q("Colecciones: agregar juegos", "collections add {1} {2}", "Nombre AppID", "Añade un juego"),
    q("Colecciones: ver", "collections show {}", "Nombre", "Juegos de la colección"),
    a("Colecciones: traer de Steam", "collections pull", "Fusiona las de Steam"),
    a("Colecciones: subir a Steam", "collections push --yes", "Sube las locales"),
    a("Colecciones: sincronizar", "collections sync --yes", "Pull y push"),
];

const SYSTEM: &[Act] = &[
    i("Iniciar sesión", "login", "Pide credenciales y Steam Guard"),
    i("Iniciar sesión con QR", "login --qr --timeout 120", "Escanea con la app móvil de Steam"),
    a("Cerrar sesión", "logout", "Borra la sesión guardada"),
    a("Cuenta", "account", "Detalles de la cuenta"),
    a("Configuración actual", "config show", "Todo lo configurado"),
    q("Idioma por defecto", "config language {}", "p. ej. spanish", "Texto de tienda y logros"),
    q("País por defecto", "config country {}", "p. ej. CO", "Región para precios"),
    q("Límite de descarga (kbit/s)", "config download-limit {}", "número u off", "Limita descargas"),
    q("Reanudar instalaciones al iniciar", "config install-resume {}", "auto | off", "Reanudación automática"),
    q("Ocultar opciones no instaladas", "config hide-uninstalled-launch-options {}", "on | off", "En launch-options"),
    a("Detener demonio", "daemon stop", "Termina el demonio de sesión"),
    a("Capacidades (JSON)", "capabilities", "Comandos, flags, errores y códigos"),
    a("Versión", "--version", "Versión de aurelia"),
    a("Ayuda de aurelia", "--help", "Lista de subcomandos"),
    i("Actualizar Aspire-Team", "self-update", "Descarga y compila la última versión desde GitHub"),
    i("Actualizar Aurelia", "update-aurelia", "Actualiza aurelia a la última versión disponible"),
];

pub fn acts_for(screen: usize) -> &'static [Act] {
    match screen {
        2 => INSTALLS,
        3 => STORE,
        4 => MODS,
        5 => PLUGINS,
        6 => SOCIAL,
        7 => SYSTEM,
        _ => &[],
    }
}

/// Busca una acción del menú de un juego por su etiqueta.
pub fn game_act(label: &str) -> Option<&'static Act> {
    GAME_ACTS.iter().find(|a| a.label == label)
}

pub fn build_args(tpl: &str, app: Option<&str>, input: &str) -> Vec<String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let nth = |n: usize| parts.get(n).copied().unwrap_or("").to_string();
    let mut out = vec![];
    for t in tpl.split_whitespace() {
        match t {
            "{app}" => out.push(app.unwrap_or("").to_string()),
            "{}" => out.push(input.trim().to_string()),
            "{..}" => out.extend(parts.iter().map(|s| s.to_string())),
            "{1}" => out.push(nth(0)),
            "{2}" => out.push(nth(1)),
            "{3}" => out.push(nth(2)),
            "{2..}" => out.push(parts.iter().skip(1).copied().collect::<Vec<_>>().join(" ")),
            _ => out.push(t.to_string()),
        }
    }
    out
}

// ---------- Modal "Jugar con opciones" ----------

pub struct LRow {
    pub label: &'static str,
    pub opts: Vec<String>,
    pub idx: usize,
    pub text: String,
    pub hint: &'static str,
}

pub struct Launch {
    pub app: String,
    pub name: String,
    pub rows: Vec<LRow>,
    pub cursor: usize,
    pub editing: bool,
}

fn lr(label: &'static str, opts: Vec<String>, hint: &'static str) -> LRow {
    LRow { label, opts, idx: 0, text: String::new(), hint }
}

impl Launch {
    pub fn new(app: String, name: String, proton_versions: &[String]) -> Self {
        let mut proton_opts = vec!["(ninguno)".to_string()];
        proton_opts.extend(proton_versions.iter().cloned());
        let rows = vec![
            lr("Plataforma", vec!["auto".into(), "linux".into(), "windows".into()], "--platform: build nativa o Windows vía Proton"),
            lr("Proton / Wine", proton_opts, "--proton <nombre>  (ver Complementos → Proton: listar)"),
            lr("Modo Steam", vec!["auto".into(), "none".into(), "host".into(), "runtime".into()], "--steam-mode: runtime=Wine, host=Steam instalado"),
            lr("Steam online", vec!["no".into(), "sí".into()], "--steam: Family Sharing / DRM"),
            lr("umu-launcher", vec!["no".into(), "sí".into()], "--umu: Proton vía umu-run"),
            lr("Motor nativo", vec!["no".into(), "sí".into()], "--native-engine: luxtorpeda"),
            lr("Shaders Vulkan", vec!["auto".into(), "omitir".into()], "--skip-shaders"),
            lr("Script de envoltura", vec!["auto".into(), "ninguno".into()], "--no-script"),
            lr("ID launch-option", vec![], "--launch-option <id>  (ver opciones de lanzamiento)"),
            lr("Script propio", vec![], "--script <ruta>"),
            lr("Detalle de logs", vec!["normal".into(), "-v".into(), "-vv".into()], "Progreso y depuración"),
            lr("Parámetros Steam", vec![], "Solo para guardar con `s`: gamemoderun %command% -novid"),
        ];
        Launch { app, name, rows, cursor: 0, editing: false }
    }

    pub fn value(&self, i: usize) -> String {
        let r = &self.rows[i];
        if r.opts.is_empty() { r.text.clone() } else { r.opts[r.idx].to_string() }
    }

    pub fn args(&self) -> Vec<String> {
        let mut a: Vec<String> = vec!["play".into(), self.app.clone()];
        let mut flag = |f: &str, v: Option<String>| {
            a.push(f.into());
            if let Some(v) = v {
                a.push(v);
            }
        };
        if self.value(0) != "auto" { flag("--platform", Some(self.value(0))) }
        if self.value(1) != "(ninguno)" && !self.value(1).is_empty() { flag("--proton", Some(self.value(1))) }
        if self.value(2) != "auto" { flag("--steam-mode", Some(self.value(2))) }
        if self.value(3) == "sí" { flag("--steam", None) }
        if self.value(4) == "sí" { flag("--umu", None) }
        if self.value(5) == "sí" { flag("--native-engine", None) }
        if self.value(6) == "omitir" { flag("--skip-shaders", None) }
        if self.value(7) == "ninguno" { flag("--no-script", None) }
        if !self.value(8).is_empty() { flag("--launch-option", Some(self.value(8))) }
        if !self.value(9).is_empty() { flag("--script", Some(self.value(9))) }
        match self.value(10).as_str() {
            "-v" => flag("-v", None),
            "-vv" => flag("-vv", None),
            _ => {}
        }
        a
    }

    /// `config game <app> [--proton P] [--launch-options L]`
    pub fn save_args(&self) -> Option<Vec<String>> {
        let mut a: Vec<String> = vec!["config".into(), "game".into(), self.app.clone()];
        if !self.value(1).is_empty() && self.value(1) != "(ninguno)" {
            a.push("--proton".into());
            a.push(self.value(1));
        }
        if !self.value(11).is_empty() {
            a.push("--launch-options".into());
            a.push(self.value(11));
        }
        if self.value(4) == "sí" { a.push("--umu".into()) }
        if self.value(5) == "sí" { a.push("--native-engine".into()) }
        (a.len() > 3).then_some(a)
    }
}
