use thiserror::Error;

#[derive(Error, Debug)]
pub enum SunabaError {
    #[error("docker error: {0}")]
    Docker(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("timeout")]
    Timeout,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for SunabaError {
    fn from(e: std::io::Error) -> Self { Self::Io(e.to_string()) }
}
