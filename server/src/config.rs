use std::path::{PathBuf};
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct Config {
    pub ws_root: PathBuf,
    pub images: Images,
    pub containers: Containers,
    pub container_ws_path: &'static str,
}

#[derive(Clone, Debug)]
pub struct Images {
    pub python: String,
    pub rust: String,
    pub go: String,
    pub bun: String,
}

#[derive(Clone, Debug)]
pub struct Containers {
    pub python: String,
    pub rust: String,
    pub go: String,
    pub bun: String,
}

pub static DEFAULT: Lazy<Config> = Lazy::new(|| Config::from_env());

impl Config {
    pub fn from_env() -> Self {
        let ws_root = std::env::var("SUNABA_WS_ROOT").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("workspace"));
        let images = Images {
            python: std::env::var("SUNABA_IMAGE_PYTHON").unwrap_or_else(|_| "python:3.12-alpine".to_string()),
            rust: std::env::var("SUNABA_IMAGE_RUST").unwrap_or_else(|_| "rust:alpine".to_string()),
            go: std::env::var("SUNABA_IMAGE_GO").unwrap_or_else(|_| "golang:alpine".to_string()),
            bun: std::env::var("SUNABA_IMAGE_BUN").unwrap_or_else(|_| "oven/bun:alpine".to_string()),
        };
        let containers = Containers {
            python: std::env::var("SUNABA_CONTAINER_PYTHON").unwrap_or_else(|_| "python_env".to_string()),
            rust: std::env::var("SUNABA_CONTAINER_RUST").unwrap_or_else(|_| "rust_env".to_string()),
            go: std::env::var("SUNABA_CONTAINER_GO").unwrap_or_else(|_| "go_env".to_string()),
            bun: std::env::var("SUNABA_CONTAINER_BUN").unwrap_or_else(|_| "bun_env".to_string()),
        };
        Self { ws_root, images, containers, container_ws_path: "/workspace" }
    }
}
