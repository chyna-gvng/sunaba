use rmcp::{tool_router, tool};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::handler::server::{ServerHandler, router::Router};

use crate::models::*;
use crate::{workspace, docker, config::DEFAULT};

#[derive(Clone)]
pub struct SunabaTools;

#[tool_router]
impl SunabaTools {
    pub fn new() -> Self { Self }

    #[tool(name = "workspace_init", description = "Create base workspaces for languages")]
    pub async fn workspace_init(&self, params: Parameters<WorkspaceInitInput>) -> Result<Json<WorkspaceInitOutput>, String> {
        let langs = params.0.languages.unwrap_or_else(|| vec![Language::Python, Language::Rust, Language::Go, Language::Bun]);
        let ids: Vec<&str> = langs.iter().map(|l| match l { Language::Python => "python", Language::Rust => "rust", Language::Go => "go", Language::Bun => "bun" }).collect();
        let (created, existing) = workspace::ensure_dirs(&ids).map_err(|e| e.to_string())?;
        Ok(Json(WorkspaceInitOutput { created, existing }))
    }

    #[tool(name = "env_start", description = "Ensure and start language container")]
    pub async fn env_start(&self, params: Parameters<EnvLangInput>) -> Result<Json<EnvStartOutput>, String> {
        let docker = docker::docker_client().map_err(|e| e.to_string())?;
        let lang = params.0.language;
        let name = docker::ensure_container(&docker, &lang).await.map_err(|e| e.to_string())?;
        docker::start_container(&docker, &name).await.map_err(|e| e.to_string())?;
        Ok(Json(EnvStartOutput { container_name: name, started: true }))
    }

    #[tool(name = "env_status", description = "Inspect container status")]
    pub async fn env_status(&self, params: Parameters<EnvLangInput>) -> Result<Json<EnvStatusOutput>, String> {
        let docker = docker::docker_client().map_err(|e| e.to_string())?;
        let lang = params.0.language;
        let name = docker::container_name_for(&lang);
        let (exists, running, pid) = docker::status(&docker, &name).await.map_err(|e| e.to_string())?;
        Ok(Json(EnvStatusOutput { container_name: name, exists, running, pid }))
    }

    #[tool(name = "env_stop", description = "Stop running container")]
    pub async fn env_stop(&self, params: Parameters<EnvStopInput>) -> Result<Json<EnvStopOutput>, String> {
        let docker = docker::docker_client().map_err(|e| e.to_string())?;
        let lang = params.0.language;
        let name = docker::container_name_for(&lang);
        let timeout = params.0.timeout_secs.unwrap_or(5);
        let _ = docker::stop_container(&docker, &name, timeout).await.map_err(|e| e.to_string())?;
        Ok(Json(EnvStopOutput { container_name: name, stopped: true }))
    }

    #[tool(name = "env_clean", description = "Remove container and optionally workspace")]
    pub async fn env_clean(&self, params: Parameters<EnvCleanInput>) -> Result<Json<EnvCleanOutput>, String> {
        let docker = docker::docker_client().map_err(|e| e.to_string())?;
        let lang = params.0.language;
        let name = docker::container_name_for(&lang);
        let _ = docker::stop_container(&docker, &name, 2).await.ok();
        let removed = docker::remove_container(&docker, &name).await.is_ok();
        let mut workspace_cleared = None;
        if params.0.remove_workspace.unwrap_or(false) {
            let slug = match lang { Language::Python => "python", Language::Rust => "rust", Language::Go => "go", Language::Bun => "bun" };
            let dir = workspace::language_dir(slug);
            if dir.exists() { let _ = std::fs::remove_dir_all(&dir); workspace_cleared = Some(true); } else { workspace_cleared = Some(false); }
        }
        Ok(Json(EnvCleanOutput { container_name: Some(name), removed, workspace_cleared }))
    }

    #[tool(name = "file_write", description = "Atomically write a file in the workspace")]
    pub async fn file_write(&self, params: Parameters<FileWriteInput>) -> Result<Json<FileWriteOutput>, String> {
        let lang_slug = match &params.0.language { Language::Python => "python", Language::Rust => "rust", Language::Go => "go", Language::Bun => "bun" };
        let path = workspace::resolve_safe_path(lang_slug, &params.0.rel_path).map_err(|e| e.to_string())?;
        let (bytes, created) = workspace::atomic_write(&path, &params.0.content, params.0.create_parents.unwrap_or(true)).map_err(|e| e.to_string())?;
        Ok(Json(FileWriteOutput { path: path.display().to_string(), bytes, created }))
    }

    #[tool(name = "exec_code", description = "Execute code inside the language container")]
    pub async fn exec_code(&self, params: Parameters<ExecCodeInput>) -> Result<Json<ExecCodeOutput>, String> {
        let input = params.0;
        let docker = docker::docker_client().map_err(|e| e.to_string())?;
        let lang = input.language.clone();
        let (lang_slug, default_file, run_cmd): (&str, &str, fn(&str, &ExecCodeInput) -> Vec<String>) = match &lang {
            Language::Python => ("python", "main.py", |ep, inp| { let mut v = vec!["python".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v }),
            Language::Rust => ("rust", "main.rs", |ep, inp| { vec!["/bin/sh".into(), "-c".into(), format!("rustc {} -o /workspace/main_bin {} && /workspace/main_bin{}", ep, inp.compile_args.as_ref().map(|v| v.join(" ")).unwrap_or_default(), inp.args.as_ref().map(|v| format!(" {}", v.join(" "))).unwrap_or_default())] }),
            Language::Go => ("go", "main.go", |ep, inp| { let mut v = vec!["go".into(), "run".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v }),
            Language::Bun => ("bun", "main.js", |ep, inp| { let mut v = vec!["bun".into(), "run".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v }),
        };
        let _ = workspace::ensure_dirs(&[lang_slug]).map_err(|e| e.to_string())?;
        let ep_rel = if let Some(ep) = &input.entrypoint { ep.clone() } else { default_file.to_string() };
        if let Some(code) = &input.code {
            let ep = workspace::resolve_safe_path(lang_slug, &ep_rel).map_err(|e| e.to_string())?;
            let _ = workspace::atomic_write(&ep, code, true).map_err(|e| e.to_string())?;
        }
        let name = docker::ensure_container(&docker, &lang).await.map_err(|e| e.to_string())?;
        let _ = docker::start_container(&docker, &name).await.map_err(|e| e.to_string())?;
        let container_ep = format!("{}/{}", DEFAULT.container_ws_path, ep_rel);
        let cmd = run_cmd(&container_ep, &input);
        let env = input.env.map(|m| m.into_iter().map(|(k,v)| format!("{}={}", k, v)).collect());
        let start = std::time::Instant::now();
        let timeout_secs = input.timeout_secs.unwrap_or(60);
        let res = docker::exec(&docker, &name, cmd, env, std::time::Duration::from_secs(timeout_secs)).await.map_err(|e| e.to_string())?;
        let out = ExecCodeOutput { exit_code: res.exit_code, stdout: res.stdout, stderr: res.stderr, duration_ms: start.elapsed().as_millis(), artifact: if matches!(lang, Language::Rust) { Some(format!("{}/{}", DEFAULT.container_ws_path, "main_bin")) } else { None }, };
        Ok(Json(out))
    }
}

impl ServerHandler for SunabaTools {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability { list_changed: Some(false) }),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: "sunaba-mcp".into(),
                title: Some("Sunaba MCP Server".into()),
                version: env!("CARGO_PKG_VERSION").into(),
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }
}

pub fn build_router() -> Router<SunabaTools> {
    Router::new(SunabaTools::new())
        .with_tools(SunabaTools::tool_router())
}
