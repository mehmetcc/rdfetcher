use rdfetcher::{logging, shared_set::SharedSet};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    logging::init_logger();

    let set = SharedSet::new();
    set.insert("mylittlepony.com").await;
    set.insert("fenasikerim.com".to_string()).await;

    let initial_size = set.len().await;
    info!(initial_size, "Initial set size");

    info!("Application started");
    Ok(())
}
