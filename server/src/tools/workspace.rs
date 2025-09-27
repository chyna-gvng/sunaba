use rmcp::{tool, handler::server::wrapper::Json};
use tracing::info;

use crate::{models::{WorkspaceInitInput, WorkspaceInitOutput}, workspace};

#[tool(name = "workspace_init", description = "Create base workspaces for languages")]
pub async fn workspace_init(params: rmcp::handler::server::wrapper::Parameters<WorkspaceInitInput>) -> Result<Json<WorkspaceInitOutput>, String> {
    let langs = params.0.languages.unwrap_or_else(|| vec![crate::models::Language::Python, crate::models::Language::Rust, crate::models::Language::Go, crate::models::Language::Bun]);
    let ids: Vec<&str> = langs.iter().map(|l| match l { crate::models::Language::Python => "python", crate::models::Language::Rust => "rust", crate::models::Language::Go => "go", crate::models::Language::Bun => "bun" }).collect();
    let (created, existing) = workspace::ensure_dirs(&ids).map_err(|e| e.to_string())?;
    info!(?created, ?existing, "workspace_init");
    Ok(Json(WorkspaceInitOutput { created, existing }))
}
