use std::time::Instant;

use rmcp::{tool, handler::server::wrapper::Json};


use crate::{models::{ExecCodeInput, ExecCodeOutput}, docker, workspace, config::DEFAULT};

#[tool(name = "exec_code", description = "Execute code inside the language container")]
pub async fn exec_code(params: rmcp::handler::server::wrapper::Parameters<ExecCodeInput>) -> Result<Json<ExecCodeOutput>, String> {
    let input = params.0;
    let docker = docker::docker_client().map_err(|e| e.to_string())?;
    let lang = input.language.clone();

    // Ensure workspace and container
    let (lang_slug, default_file, run_cmd): (&str, &str, fn(&str, &ExecCodeInput) -> Vec<String>) = match &lang {
        crate::models::Language::Python => ("python", "main.py", |ep, inp| {
            let mut v = vec!["python".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v
        }),
        crate::models::Language::Rust => ("rust", "main.rs", |ep, inp| {
            // Compile then run binary /workspace/main_bin
            vec!["/bin/sh".into(), "-c".into(), format!("rustc {} -o /workspace/main_bin {} && /workspace/main_bin{}",
                ep,
                inp.compile_args.as_ref().map(|v| v.join(" ")).unwrap_or_default(),
                inp.args.as_ref().map(|v| format!(" {}", v.join(" "))).unwrap_or_default()
            )]
        }),
        crate::models::Language::Go => ("go", "main.go", |ep, inp| {
            let mut v = vec!["go".into(), "run".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v
        }),
        crate::models::Language::Bun => ("bun", "main.js", |ep, inp| {
            let mut v = vec!["bun".into(), "run".into(), ep.into()]; if let Some(a)=&inp.args { v.extend(a.clone()); } v
        }),
    };

    // Ensure workspace dir exists
    let _ = workspace::ensure_dirs(&[lang_slug]).map_err(|e| e.to_string())?;

    // Determine entrypoint
    let ep_rel = if let Some(ep) = &input.entrypoint { ep.clone() } else { default_file.to_string() };
    // If provided code, write it
    if let Some(code) = &input.code {
        let ep = workspace::resolve_safe_path(lang_slug, &ep_rel).map_err(|e| e.to_string())?;
        let _ = workspace::atomic_write(&ep, code, true).map_err(|e| e.to_string())?;
    }

    // Ensure container running
    let name = docker::ensure_container(&docker, &lang).await.map_err(|e| e.to_string())?;
    let _ = docker::start_container(&docker, &name).await.map_err(|e| e.to_string())?;

    // Build command
    let container_ep = format!("{}/{}", DEFAULT.container_ws_path, ep_rel);
    let cmd = run_cmd(&container_ep, &input);
    let env = input.env.map(|m| m.into_iter().map(|(k,v)| format!("{}={}", k, v)).collect());

    let start = Instant::now();
    let timeout_secs = input.timeout_secs.unwrap_or(60);
    let res = docker::exec(&docker, &name, cmd, env, std::time::Duration::from_secs(timeout_secs))
        .await
        .map_err(|e| e.to_string())?;

    let out = ExecCodeOutput {
        exit_code: res.exit_code,
        stdout: res.stdout,
        stderr: res.stderr,
        duration_ms: start.elapsed().as_millis(),
        artifact: if matches!(lang, crate::models::Language::Rust) { Some(format!("{}/{}", DEFAULT.container_ws_path, "main_bin")) } else { None },
    };
    Ok(Json(out))
}
