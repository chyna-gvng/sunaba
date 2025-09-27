use std::time::Duration;

use bollard::Docker;
use bollard::models::{HostConfig, ContainerCreateBody};
use bollard::query_parameters::{CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions, InspectContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::query_parameters::CreateImageOptionsBuilder;
use futures::StreamExt;
use tokio::time::timeout;

use crate::{config::DEFAULT, errors::SunabaError};

pub fn docker_client() -> Result<Docker, SunabaError> {
    #[cfg(unix)]
    { Docker::connect_with_socket_defaults().map_err(|e| SunabaError::Docker(e.to_string())) }
    #[cfg(not(unix))]
    { Docker::connect_with_local_defaults().map_err(|e| SunabaError::Docker(e.to_string())) }
}

fn names_for(lang: &crate::models::Language) -> (String, String, String) {
    let (container, image, ws_lang) = match lang {
        crate::models::Language::Python => (&DEFAULT.containers.python, &DEFAULT.images.python, "python"),
        crate::models::Language::Rust => (&DEFAULT.containers.rust, &DEFAULT.images.rust, "rust"),
        crate::models::Language::Go => (&DEFAULT.containers.go, &DEFAULT.images.go, "go"),
        crate::models::Language::Bun => (&DEFAULT.containers.bun, &DEFAULT.images.bun, "bun"),
    };
    (container.clone(), image.clone(), ws_lang.to_string())
}

async fn ensure_image(docker: &Docker, image: &str) -> Result<(), SunabaError> {
    if docker.inspect_image(image).await.is_ok() {
        return Ok(());
    }
    let (from_image, tag) = match image.rsplit_once(':') {
        Some((name, tag)) if !tag.is_empty() => (name, tag),
        _ => (image, "latest"),
    };
    let opts = CreateImageOptionsBuilder::default()
        .from_image(from_image)
        .tag(tag)
        .build();
    let mut stream = docker.create_image(
        Some(opts),
        None,
        None,
    );
    while let Some(next) = stream.next().await {
        match next {
            Ok(_chunk) => { /* ignore progress */ }
            Err(e) => return Err(SunabaError::Docker(e.to_string())),
        }
    }
    Ok(())
}

pub async fn ensure_container(docker: &Docker, lang: &crate::models::Language) -> Result<String, SunabaError> {
    let (name, image, ws_lang) = names_for(lang);
    if docker.inspect_container(&name, None::<InspectContainerOptions>).await.is_ok() {
        return Ok(name);
    }

    // Ensure the image is available locally
    ensure_image(docker, &image).await?;

    // Ensure workspace directory exists on host for bind mount
    let _ = crate::workspace::ensure_dirs(&[&ws_lang])?;
    let host_ws = std::fs::canonicalize(crate::workspace::language_dir(&ws_lang)).unwrap_or(DEFAULT.ws_root.join(&ws_lang));
    let container_ws = DEFAULT.container_ws_path;

    let binds = vec![ format!("{}:{}", host_ws.display(), container_ws) ];
    let host_config = HostConfig { binds: Some(binds), ..Default::default() };

    let cfg = ContainerCreateBody {
        image: Some(image),
        tty: Some(true),
        open_stdin: Some(true),
        cmd: Some(vec!["/bin/sh".into(), "-c".into(), "sleep infinity".into()]),
        host_config: Some(host_config),
        ..Default::default()
    };
    let opts = CreateContainerOptions { name: Some(name.clone()), platform: String::new() };
    docker.create_container(Some(opts), cfg)
        .await
        .map_err(|e| SunabaError::Docker(e.to_string()))?;
    Ok(name)
}

pub async fn start_container(docker: &Docker, name: &str) -> Result<(), SunabaError> {
    docker.start_container(name, None::<StartContainerOptions>)
        .await
        .map_err(|e| SunabaError::Docker(e.to_string()))?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, name: &str, timeout_secs: u64) -> Result<(), SunabaError> {
    docker.stop_container(name, Some(StopContainerOptions { t: Some((timeout_secs as i64).try_into().unwrap()), ..Default::default() }))
        .await
        .map_err(|e| SunabaError::Docker(e.to_string()))?;
    Ok(())
}

pub async fn remove_container(docker: &Docker, name: &str) -> Result<(), SunabaError> {
    docker.remove_container(name, Some(RemoveContainerOptions { force: true, ..Default::default() }))
        .await
        .map_err(|e| SunabaError::Docker(e.to_string()))?;
    Ok(())
}

pub async fn status(docker: &Docker, name: &str) -> Result<(bool, bool, Option<i64>), SunabaError> {
    match docker.inspect_container(name, None::<InspectContainerOptions>).await {
        Ok(details) => {
            let running = details.state.as_ref().and_then(|s| s.running).unwrap_or(false);
            let pid = details.state.as_ref().and_then(|s| s.pid);
            Ok((true, running, pid))
        }
        Err(_) => Ok((false, false, None)),
    }
}

pub struct ExecResult { pub exit_code: i64, pub stdout: String, pub stderr: String }

pub async fn exec(docker: &Docker, name: &str, cmd: Vec<String>, env: Option<Vec<String>>, time_limit: Duration) -> Result<ExecResult, SunabaError> {
    let create = docker.create_exec(name, CreateExecOptions {
        attach_stdout: Some(true), attach_stderr: Some(true), attach_stdin: Some(false),
        tty: Some(false),
        cmd: Some(cmd.clone()),
        env,
        ..Default::default()
    }).await.map_err(|e| SunabaError::Docker(e.to_string()))?;

    let id = create.id;

    let fut = async {
        match docker.start_exec(&id, Some(StartExecOptions { detach: false, tty: false, output_capacity: None })).await {
            Ok(StartExecResults::Attached { mut output, .. }) => {
                let mut out = String::new();
                let mut err = String::new();
                while let Some(Ok(msg)) = output.next().await {
                    use bollard::container::LogOutput;
                    match msg {
                        LogOutput::StdOut { message } => { out.push_str(&String::from_utf8_lossy(&message)); }
                        LogOutput::StdErr { message } => { err.push_str(&String::from_utf8_lossy(&message)); }
                        LogOutput::Console { message } => { out.push_str(&String::from_utf8_lossy(&message)); }
                        _ => {}
                    }
                }
                // Inspect exec for exit code
                let inspected = docker.inspect_exec(&id).await.map_err(|e| SunabaError::Docker(e.to_string()))?;
                let code = inspected.exit_code.unwrap_or_default();
                Ok(ExecResult { exit_code: code as i64, stdout: out, stderr: err })
            }
            Ok(_) => Err(SunabaError::Internal("unexpected exec result".into())),
            Err(e) => Err(SunabaError::Docker(e.to_string())),
        }
    };

    match timeout(time_limit, fut).await {
        Ok(res) => res,
        Err(_) => Err(SunabaError::Timeout),
    }
}

pub fn container_name_for(lang: &crate::models::Language) -> String { names_for(lang).0 }
