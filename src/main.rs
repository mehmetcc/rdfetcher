use rdfetcher::{
    logging,
    page::{AbsoluteUrl, Page},
    shared_set::SharedSet,
};
use tracing::info;

const INITIAL_URL: &str = "https://doc.rust-lang.org/stable/std";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    logging::init_logger();

    let set = SharedSet::new();
    set.insert(INITIAL_URL).await;

    let initial_size = set.len().await;
    info!(initial_size, "Initial set size");

    let initial_page = Page::new(AbsoluteUrl::new(INITIAL_URL, None)).await?;

    // testing here. delet in the future
    let page = Page::new(AbsoluteUrl::new(INITIAL_URL, None)).await?;
    for link in page.extract_links().await?.iter() {
        info!("{}", link);
    }
    // testing here. delet in the future

    info!("Application started");
    Ok(())
}
