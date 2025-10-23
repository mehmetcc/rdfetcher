use rdfetcher::{logging, page::AbsoluteUrl, spider::Spider};
use tracing::info;

const INITIAL_URL: &str = "https://doc.rust-lang.org/stable/std";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    logging::init_logger();

    let initial_url = AbsoluteUrl::new(INITIAL_URL, None);
    let spider = Spider::new(initial_url).await?;
    let _ = spider.start().await;

    info!("Application started");
    Ok(())
}
