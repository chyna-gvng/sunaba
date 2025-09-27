use std::{fs, io::Write, path::{Path, PathBuf}};

use crate::{config::DEFAULT, errors::SunabaError};

pub fn language_dir(lang: &str) -> PathBuf {
    DEFAULT.ws_root.join(lang)
}

pub fn ensure_dirs(langs: &[&str]) -> Result<(Vec<String>, Vec<String>), SunabaError> {
    let mut created = Vec::new();
    let mut existing = Vec::new();
    for l in langs {
        let p = language_dir(l);
        if p.exists() { existing.push(p.display().to_string()); } else { fs::create_dir_all(&p)?; created.push(p.display().to_string()); }
    }
    Ok((created, existing))
}

pub fn resolve_safe_path(lang: &str, rel_path: &str) -> Result<PathBuf, SunabaError> {
    let base = language_dir(lang);
    let target = base.join(rel_path);
    let canon_base = std::fs::canonicalize(&base).unwrap_or(base);
    let canon_target = std::fs::canonicalize(&target).unwrap_or(target.clone());
    if !canon_target.starts_with(&canon_base) {
        return Err(SunabaError::InvalidInput("path escapes workspace".into()));
    }
    Ok(target)
}

pub fn atomic_write(path: &Path, content: &str, create_parents: bool) -> Result<(usize, bool), SunabaError> {
    if let Some(parent) = path.parent() {
        if create_parents { std::fs::create_dir_all(parent)?; }
    }
    let tmp = path.with_extension(".tmp.snb");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(content.as_bytes())?;
        f.sync_all()?;
    }
    let existed = path.exists();
    std::fs::rename(&tmp, path)?;
    Ok((content.len(), !existed))
}
