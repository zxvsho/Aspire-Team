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

/// Menú de un juego (Enter o `a` en Biblioteca).
pub const GAME_ACTS: &[Act] = &[
    g("Jugar", "play {app}", "Lanza el juego con la configuración guardada"),
    m("Jugar con opciones…", Mode::Launch, None, "Plataforma, Proton, umu, scripts, parámetros de lanzamiento"),
    g("Instalar", "install {app}", "Descarga e instala el juego"),
    g("Actualizar", "update {app}", "Descarga el último manifiesto"),
    g("Verificar archivos", "verify {app}", "Comprueba la integridad de los archivos"),
    g("Desinstalar", "uninstall {app}", "Elimina el juego"),
    g("Detener", "stop {app}", "Detiene el juego en ejecución"),
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
    m("Aplicar parche/mod local (carpeta)", Mode::Patch, Some("Carpeta del parche o mod"), "Copia la carpeta sobre el juego con respaldo"),
    m("Revertir último parche local", Mode::Revert, None, "Restaura los archivos respaldados"),
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
    q("Buscar juego", "search {}", "Título", "Busca AppIDs por título"),
    a("Ofertas", "deals", "Más vendidos con descuento"),
    a("Eventos y rebajas", "events", "Eventos de la tienda"),
    q("Información", "info {}", "AppID", "Detalles del juego"),
    q("Información extendida", "info {} --extended", "AppID", "Requisitos, géneros…"),
    q("Precio por región", "price {} --compare US,DE,JP", "AppID", "Compara regiones"),
    q("Similares", "similar {}", "AppID", "Juegos parecidos"),
    q("Jugadores ahora", "players {}", "AppID", "Jugadores conectados"),
    q("Noticias", "news {}", "AppID", "Anuncios recientes"),
    q("Reseñas", "reviews {}", "AppID", "Reseñas de usuarios"),
    q("Reseñas negativas", "reviews {} -t negative", "AppID", "Solo negativas"),
    q("DLC", "dlc {}", "AppID", "DLC y propiedad"),
    q("Logros", "achievements {}", "AppID", "Tus logros"),
    q("Verificar DRM/propiedad", "drm {}", "AppID", "Tickets sin cliente Steam"),
    q("Portada (caché)", "image {}", "AppID", "Descarga arte y muestra la ruta"),
    q("Imágenes y trailers", "image {} --list --probe", "AppID", "URLs de arte, capturas y trailers"),
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
    m("Aplicar parche/mod local (carpeta)", Mode::Patch, Some("Carpeta del parche o mod"), "Requiere juego elegido en Biblioteca; con respaldo"),
    m("Revertir último parche local", Mode::Revert, None, "Requiere juego elegido en Biblioteca"),
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
    q("Proton: instalar", "proton install {}", "Nombre (GE-Proton9-20, Proton-CachyOS…)", "Descarga un runtime"),
    q("Proton: predeterminado", "proton default {}", "Nombre", "Runtime global por defecto"),
    q("Proton: desinstalar", "proton uninstall {}", "Nombre", "Borra un runtime"),
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
    q("Steam runtime: elegir runner", "config steam-runtime-runner {}", "Runner (p. ej. GE-Proton9-20)", "Requerido antes de instalar"),
    i("Steam runtime: instalar", "steam-runtime install", "Instala Steam en Wine (inicias sesión ahí)"),
    i("Steam runtime: iniciar sesión", "steam-runtime login", "Abre Steam en Wine"),
    i("Steam runtime: reparar", "steam-runtime repair", "Respalda y reinstala"),
    a("Steam runtime: detener", "steam-runtime stop", "Cierra Steam en Wine"),
    a("Steam runtime: desinstalar", "steam-runtime uninstall", "Borra el prefijo"),
    q("Steam runtime: política", "config steam-runtime-policy {}", "auto | on | off", "Cuándo usar el Steam en Wine"),
    q("Shader precache", "config shader-precache {}", "auto | off", "Precompilar shaders Vulkan"),
    q("Precompilar shaders de un juego", "shaders {}", "AppID", "Linux: ahora mismo"),
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
    pub opts: &'static [&'static str],
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

fn lr(label: &'static str, opts: &'static [&'static str], hint: &'static str) -> LRow {
    LRow { label, opts, idx: 0, text: String::new(), hint }
}

impl Launch {
    pub fn new(app: String, name: String) -> Self {
        let rows = vec![
            lr("Plataforma", &["auto", "linux", "windows"], "--platform: build nativa o Windows vía Proton"),
            lr("Proton / Wine", &[], "--proton <nombre>  (ver Complementos → Proton)"),
            lr("Modo Steam", &["auto", "none", "host", "runtime"], "--steam-mode: runtime=Wine, host=Steam instalado"),
            lr("Steam online", &["no", "sí"], "--steam: Family Sharing / DRM"),
            lr("umu-launcher", &["no", "sí"], "--umu: Proton vía umu-run"),
            lr("Motor nativo", &["no", "sí"], "--native-engine: luxtorpeda"),
            lr("Shaders Vulkan", &["auto", "omitir"], "--skip-shaders"),
            lr("Script de envoltura", &["auto", "ninguno"], "--no-script"),
            lr("ID launch-option", &[], "--launch-option <id>  (ver opciones de lanzamiento)"),
            lr("Script propio", &[], "--script <ruta>"),
            lr("Detalle de logs", &["normal", "-v", "-vv"], "Progreso y depuración"),
            lr("Parámetros Steam", &[], "Solo para guardar con `s`: gamemoderun %command% -novid"),
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
        if !self.value(1).is_empty() { flag("--proton", Some(self.value(1))) }
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
        if !self.value(1).is_empty() {
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
