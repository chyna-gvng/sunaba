use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Language { Python, Rust, Go, Bun }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceInitInput { pub languages: Option<Vec<Language>> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceInitOutput { pub created: Vec<String>, pub existing: Vec<String> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvLangInput { pub language: Language }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvStartOutput { pub container_name: String, pub started: bool }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvStatusOutput { pub container_name: String, pub exists: bool, pub running: bool, pub pid: Option<i64> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvStopInput { pub language: Language, pub timeout_secs: Option<u64> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvStopOutput { pub container_name: String, pub stopped: bool }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvCleanInput { pub language: Language, pub remove_workspace: Option<bool> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvCleanOutput { pub container_name: Option<String>, pub removed: bool, pub workspace_cleared: Option<bool> }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileWriteInput {
    pub language: Language,
    pub rel_path: String,
    pub content: String,
    pub create_parents: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileWriteOutput { pub path: String, pub bytes: usize, pub created: bool }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecCodeInput {
    pub language: Language,
    pub entrypoint: Option<String>,
    pub code: Option<String>,
    pub args: Option<Vec<String>>,
    pub compile_args: Option<Vec<String>>,
    pub timeout_secs: Option<u64>,
    pub env: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecCodeOutput {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u128,
    pub artifact: Option<String>,
}
