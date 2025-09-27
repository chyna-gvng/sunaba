use rmcp::{tool, handler::server::wrapper::Json};
use crate::{models::{FileWriteInput, FileWriteOutput}, workspace};

#[tool(name = "file_write", description = "Atomically write a file in the workspace")]
pub async fn file_write(params: rmcp::handler::server::wrapper::Parameters<FileWriteInput>) -> Result<Json<FileWriteOutput>, String> {
    let (lang_slug, _lang) = match &params.0.language {
        crate::models::Language::Python => ("python", &params.0.language),
        crate::models::Language::Rust => ("rust", &params.0.language),
        crate::models::Language::Go => ("go", &params.0.language),
        crate::models::Language::Bun => ("bun", &params.0.language),
    };
    let path = workspace::resolve_safe_path(lang_slug, &params.0.rel_path).map_err(|e| e.to_string())?;
    let (bytes, created) = workspace::atomic_write(&path, &params.0.content, params.0.create_parents.unwrap_or(true)).map_err(|e| e.to_string())?;
    Ok(Json(FileWriteOutput { path: path.display().to_string(), bytes, created }))
}
