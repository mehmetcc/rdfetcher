use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};

use anyhow::anyhow;

use crate::{
    config::Config,
    page::{AbsoluteUrl, Page},
    visited_urls::VisitedUrls,
};

pub struct Spider {
    pub visited_urls: VisitedUrls,
    semaphore: Arc<Semaphore>,
}

impl Spider {
    pub async fn new(initial_url: AbsoluteUrl) -> anyhow::Result<Self> {
        let visited_urls = VisitedUrls::new();
        let inserted = visited_urls.insert_pending(initial_url).await;
        if !inserted {
            return Err(anyhow!("Can't load initial url into visited urls"));
        }

        let config = Config::load()?;
        let permits = config.concurrency.mask_tasks;

        Ok(Spider {
            visited_urls: visited_urls,
            semaphore: Arc::new(Semaphore::new(permits)),
        })
    }

    pub async fn start(&self, tx: Sender<Page>) -> anyhow::Result<()> {
        loop {
            let pending_count = self.visited_urls.pending_count().await;
            if pending_count == 0 {
                info!("No more pending URLs to process. Spider finished.");
                break;
            }

            let pending_urls = self.visited_urls.get_pending_urls().await;
            let tasks: Vec<_> = pending_urls
                .into_iter()
                .map(|link| {
                    let semaphore = self.semaphore.clone();
                    let visited_urls = self.visited_urls.clone();
                    let local_tx = tx.clone();

                    tokio::spawn(async move {
                        let _permit = semaphore.acquire().await?;

                        match Page::new(link.clone()).await {
                            Ok(p) => {
                                let absolute_url = p.url.full_url()?;

                                // Mark URL as visited before processing
                                visited_urls.mark_visited(link.clone()).await;
                                let _ = local_tx.send(p.clone()).await?;

                                match p.extract_links().await {
                                    Ok(found_links) => {
                                        let mut new_links_count = 0;
                                        for found_link in found_links {
                                            if visited_urls.insert_pending(found_link).await {
                                                new_links_count += 1;
                                            }
                                        }
                                        debug!(
                                            "Fetched {} - discovered {} new links",
                                            absolute_url, new_links_count
                                        );
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to extract links from {}: {:?}",
                                            absolute_url, e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                let absolute_url = link.full_url()?;
                                error!("Failed to fetch {}: {:?}", absolute_url, e);
                                visited_urls.mark_visited(link).await;
                            }
                        }

                        Ok::<(), anyhow::Error>(())
                    })
                })
                .collect();

            for task in tasks {
                if let Err(e) = task.await {
                    error!("Worker task failed to join: {:?}", e);
                }
            }
        }

        Ok(())
    }
}
