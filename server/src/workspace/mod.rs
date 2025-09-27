use std::{fs, io::Write, path::{Path, PathBuf}};

pub mod path;

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

    // Allow container-absolute paths like "/workspace/<...>" by stripping the prefix.
    let mut rel = rel_path;
    if let Some(prefix) = rel_path.strip_prefix(DEFAULT.container_ws_path) {
        // Strip leading slash if present after prefix
        rel = prefix.strip_prefix('/').unwrap_or(prefix);
    }

    // Reject absolute host paths outright (only container-absolute under /workspace is allowed)
    let path_in = Path::new(rel);
    if Path::new(rel_path).is_absolute() && !rel_path.starts_with(DEFAULT.container_ws_path) {
        return Err(SunabaError::InvalidInput("absolute paths outside container workspace are not allowed".into()));
    }

    // Lexically normalize to prevent traversal outside base
    let mut sanitized = PathBuf::new();
    for comp in path_in.components() {
        use std::path::Component;
        match comp {
            Component::CurDir => { /* skip */ }
            Component::ParentDir => {
                if !sanitized.pop() {
                    return Err(SunabaError::InvalidInput("path escapes workspace".into()));
                }
            }
            Component::Normal(seg) => sanitized.push(seg),
            Component::RootDir | Component::Prefix(_) => {
                // Should not occur due to checks above
                return Err(SunabaError::InvalidInput("invalid absolute path".into()));
            }
        }
    }

    let target = base.join(sanitized);

    // Final defensive check using canonicalize if base exists
    let canon_base = std::fs::canonicalize(&base).unwrap_or(base);
    let canon_target = match std::fs::canonicalize(&target) {
        Ok(p) => p,
        Err(_) => target.clone(), // If it doesn't exist yet, rely on lexical check above
    };
    if canon_target.is_absolute() && !canon_target.starts_with(&canon_base) {
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
