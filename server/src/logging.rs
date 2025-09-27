use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Log to stderr to keep stdout clean for MCP stdio
    let fmt_layer = fmt::layer().with_writer(std::io::stderr).with_target(true).with_ansi(true);
    tracing_subscriber::registry().with(filter).with(fmt_layer).init();
}
