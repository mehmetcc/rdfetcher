use tokio::sync::mpsc::{self};

use rdfetcher::{
    config::Config,
    kafka::KafkaSink,
    logging,
    page::{AbsoluteUrl, Page},
    spider::Spider,
};
use tracing::info;

const INITIAL_URL: &str = "https://doc.rust-lang.org/stable/std";

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    logging::init_logger();

    let config = Config::load()?;

    let (tx, rx) = mpsc::channel::<Page>(config.concurrency.buffer_size);

    let initial_url = AbsoluteUrl::new(INITIAL_URL, None);

    let spider = Spider::new(initial_url).await?;
    let kafka = KafkaSink::new()?;

    let _ = tokio::join!(spider.start(tx), kafka.dispatch(rx));

    info!("Application started");
    Ok(())
}
