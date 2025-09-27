use rmcp::{tool, handler::server::wrapper::Json};
use tracing::info;

use crate::{models::{EnvLangInput, EnvStartOutput, EnvStatusOutput, EnvStopInput, EnvStopOutput, EnvCleanInput, EnvCleanOutput}, docker};

#[tool(name = "env_start", description = "Ensure and start language container")]
pub async fn env_start(params: rmcp::handler::server::wrapper::Parameters<EnvLangInput>) -> Result<Json<EnvStartOutput>, String> {
    let docker = docker::docker_client().map_err(|e| e.to_string())?;
    let lang = params.0.language;
    let name = docker::ensure_container(&docker, &lang).await.map_err(|e| e.to_string())?;
    docker::start_container(&docker, &name).await.map_err(|e| e.to_string())?;
    info!(container = %name, "env_start");
    Ok(Json(EnvStartOutput { container_name: name, started: true }))
}

#[tool(name = "env_status", description = "Inspect container status")]
pub async fn env_status(params: rmcp::handler::server::wrapper::Parameters<EnvLangInput>) -> Result<Json<EnvStatusOutput>, String> {
    let docker = docker::docker_client().map_err(|e| e.to_string())?;
    let lang = params.0.language;
    let name = docker::container_name_for(&lang);
    let (exists, running, pid) = docker::status(&docker, &name).await.map_err(|e| e.to_string())?;
    Ok(Json(EnvStatusOutput { container_name: name, exists, running, pid }))
}

#[tool(name = "env_stop", description = "Stop running container")]
pub async fn env_stop(params: rmcp::handler::server::wrapper::Parameters<EnvStopInput>) -> Result<Json<EnvStopOutput>, String> {
    let docker = docker::docker_client().map_err(|e| e.to_string())?;
    let lang = params.0.language;
    let name = docker::container_name_for(&lang);
    let timeout = params.0.timeout_secs.unwrap_or(5);
    let _ = docker::stop_container(&docker, &name, timeout).await.map_err(|e| e.to_string())?;
    Ok(Json(EnvStopOutput { container_name: name, stopped: true }))
}

#[tool(name = "env_clean", description = "Remove container and optionally workspace")]
pub async fn env_clean(params: rmcp::handler::server::wrapper::Parameters<EnvCleanInput>) -> Result<Json<EnvCleanOutput>, String> {
    let docker = docker::docker_client().map_err(|e| e.to_string())?;
    let lang = params.0.language;
    let name = docker::container_name_for(&lang);
    let _ = docker::stop_container(&docker, &name, 2).await.ok();
    let removed = docker::remove_container(&docker, &name).await.is_ok();
    let mut workspace_cleared = None;
    if params.0.remove_workspace.unwrap_or(false) {
        let slug = match lang { crate::models::Language::Python => "python", crate::models::Language::Rust => "rust", crate::models::Language::Go => "go", crate::models::Language::Bun => "bun" };
        let dir = crate::workspace::language_dir(slug);
        if dir.exists() { let _ = std::fs::remove_dir_all(&dir); workspace_cleared = Some(true); }
        else { workspace_cleared = Some(false); }
    }
    Ok(Json(EnvCleanOutput { container_name: Some(name), removed, workspace_cleared }))
}
