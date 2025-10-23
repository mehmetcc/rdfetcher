use rdfetcher::{
    logging,
    page::{AbsoluteUrl, Page},
    visited_urls::VisitedUrls,
};
use tracing::info;

const INITIAL_URL: &str = "https://doc.rust-lang.org/stable/std";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    logging::init_logger();

    let visited_urls = VisitedUrls::new();
    visited_urls
        .insert_pending(AbsoluteUrl::new(INITIAL_URL, None))
        .await;

    let initial_size = visited_urls.len().await;
    info!(initial_size, "Initial visited URLs size");

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
