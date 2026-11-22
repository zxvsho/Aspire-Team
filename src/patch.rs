use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const BK: &str = ".aspire-backup";

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            walk(&p, out)?;
        } else {
            out.push(p);
        }
    }
    Ok(())
}

/// Copia el contenido de `src` sobre `dst`, guardando respaldo de lo sobrescrito.
pub fn apply(src: &Path, dst: &Path, log: &dyn Fn(String)) -> Result<String> {
    if !src.is_dir() {
        bail!("la carpeta del parche no existe: {}", src.display());
    }
    if !dst.is_dir() {
        bail!("la carpeta del juego no existe: {}", dst.display());
    }
    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let bk = dst.join(BK).join(ts.to_string());
    fs::create_dir_all(&bk)?;
    let mut files = vec![];
    walk(src, &mut files)?;
    let mut added = vec![];
    for f in &files {
        let rel = f.strip_prefix(src)?;
        let target = dst.join(rel);
        if target.exists() {
            let b = bk.join(rel);
            fs::create_dir_all(b.parent().unwrap())?;
            fs::copy(&target, &b).with_context(|| format!("respaldando {}", target.display()))?;
        } else {
            added.push(rel.to_string_lossy().to_string());
        }
        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(f, &target).with_context(|| format!("copiando {}", rel.display()))?;
        log(format!("✔ {}", rel.display()));
    }
    fs::write(bk.join("added.txt"), added.join("\n"))?;
    Ok(format!(
        "Parche aplicado: {} archivos ({} nuevos). Respaldo: {}\nOjo: `aurelia verify` restaurará los archivos originales.",
        files.len(),
        added.len(),
        bk.display()
    ))
}

/// Revierte el último parche aplicado en `dst`.
pub fn revert(dst: &Path, log: &dyn Fn(String)) -> Result<String> {
    let root = dst.join(BK);
    let mut dirs: Vec<PathBuf> = fs::read_dir(&root).map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).collect()).unwrap_or_default();
    dirs.sort();
    let Some(last) = dirs.pop() else { bail!("no hay parches que revertir en {}", dst.display()) };
    let mut files = vec![];
    walk(&last, &mut files)?;
    for f in &files {
        let rel = f.strip_prefix(&last)?;
        if rel == Path::new("added.txt") {
            continue;
        }
        fs::copy(f, dst.join(rel))?;
        log(format!("↩ restaurado {}", rel.display()));
    }
    if let Ok(added) = fs::read_to_string(last.join("added.txt")) {
        for rel in added.lines().filter(|l| !l.is_empty()) {
            let _ = fs::remove_file(dst.join(rel));
            log(format!("✖ eliminado {rel}"));
        }
    }
    fs::remove_dir_all(&last)?;
    Ok("Último parche revertido.".into())
}
