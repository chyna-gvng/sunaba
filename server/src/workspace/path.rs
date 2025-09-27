use std::path::{Component, Path, PathBuf};

use crate::{config::DEFAULT, errors::SunabaError};

#[derive(Clone, Debug)]
pub struct PathSpec {
    pub rel: PathBuf,
    pub host: PathBuf,
    pub container: String,
}

fn lang_slug(lang: &str) -> &str { lang }

pub fn resolve(lang: &str, input: &str) -> Result<PathSpec, SunabaError> {
    let base = super::language_dir(lang_slug(lang));

    // Compute host workspace prefixes (absolute and relative) for matching
    let host_ws = base.clone();
    let host_ws_abs = std::fs::canonicalize(&host_ws).unwrap_or(host_ws.clone());
    let host_ws_abs_str = host_ws_abs.to_string_lossy();
    let host_ws_rel_str = format!("{}/{}", DEFAULT.ws_root.display(), lang);

    // Determine a relative component list from input
    let rel_str = if input.starts_with(DEFAULT.container_ws_path) {
        let tail = &input[DEFAULT.container_ws_path.len()..];
        tail.strip_prefix('/').unwrap_or(tail)
    } else if input.starts_with(&*host_ws_abs_str) {
        let mut tail = &input[host_ws_abs_str.len()..];
        tail = tail.strip_prefix('/').unwrap_or(tail);
        tail
    } else if input.starts_with(&host_ws_rel_str) {
        let mut tail = &input[host_ws_rel_str.len()..];
        tail = tail.strip_prefix('/').unwrap_or(tail);
        tail
    } else if input.starts_with(&format!("{}/", lang)) {
        // Accept inputs like "bun/main.js" and strip the language folder
        &input[lang.len()+1..]
    } else if Path::new(input).is_absolute() {
        return Err(SunabaError::InvalidInput("absolute paths outside container workspace are not allowed".into()));
    } else {
        input
    };

    // Lexically normalize
    let mut rel = PathBuf::new();
    for comp in Path::new(rel_str).components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if !rel.pop() {
                    return Err(SunabaError::InvalidInput("path escapes workspace".into()));
                }
            }
            Component::Normal(seg) => rel.push(seg),
            Component::RootDir | Component::Prefix(_) => {
                return Err(SunabaError::InvalidInput("invalid absolute path".into()));
            }
        }
    }

    let host = base.join(&rel);
    let container = format!("{}/{}", DEFAULT.container_ws_path, rel.to_string_lossy());

    Ok(PathSpec { rel, host, container })
}
