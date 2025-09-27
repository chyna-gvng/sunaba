use server::{logging, tools};
use rmcp::service::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init();

    let router = tools::build_router();

    let running = router
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;

    // Keep the server alive until interrupted, then cancel gracefully
    let _ = tokio::signal::ctrl_c().await;
    let _ = running.cancel().await;
    Ok(())
}
