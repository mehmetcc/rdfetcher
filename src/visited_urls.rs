use crate::page::AbsoluteUrl;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct VisitedUrls {
    inner: Arc<RwLock<HashMap<AbsoluteUrl, bool>>>,
}

impl VisitedUrls {
    pub fn new() -> Self {
        VisitedUrls {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert_visited(&self, url: AbsoluteUrl) -> bool {
        let mut map = self.inner.write().await;
        if map.contains_key(&url) {
            false
        } else {
            map.insert(url, true);
            true
        }
    }

    pub async fn insert_pending(&self, url: AbsoluteUrl) -> bool {
        let mut map = self.inner.write().await;
        if map.contains_key(&url) {
            false
        } else {
            map.insert(url, false);
            true
        }
    }

    pub async fn contains(&self, url: &AbsoluteUrl) -> bool {
        let map = self.inner.read().await;
        map.contains_key(url)
    }

    pub async fn is_visited(&self, url: &AbsoluteUrl) -> bool {
        let map = self.inner.read().await;
        map.get(url).copied().unwrap_or(false)
    }

    pub async fn mark_visited(&self, url: AbsoluteUrl) -> bool {
        let mut map = self.inner.write().await;
        if let Some(visited) = map.get_mut(&url) {
            *visited = true;
            true
        } else {
            false
        }
    }

    pub async fn remove(&self, url: &AbsoluteUrl) -> bool {
        let mut map = self.inner.write().await;
        map.remove(url).is_some()
    }

    pub async fn len(&self) -> usize {
        let map = self.inner.read().await;
        map.len()
    }

    pub async fn visited_count(&self) -> usize {
        let map = self.inner.read().await;
        map.values().filter(|&&visited| visited).count()
    }

    pub async fn pending_count(&self) -> usize {
        let map = self.inner.read().await;
        map.values().filter(|&&visited| !visited).count()
    }

    pub async fn get_pending_urls(&self) -> Vec<AbsoluteUrl> {
        let map = self.inner.read().await;
        map.iter()
            .filter(|(_, visited)| !**visited)
            .map(|(url, _)| url.clone())
            .collect()
    }

    pub async fn get_visited_urls(&self) -> Vec<AbsoluteUrl> {
        let map = self.inner.read().await;
        map.iter()
            .filter(|(_, visited)| **visited)
            .map(|(url, _)| url.clone())
            .collect()
    }

    pub async fn clear(&self) {
        let mut map = self.inner.write().await;
        map.clear();
    }

    pub async fn pop_pending(&self) -> Option<AbsoluteUrl> {
        let mut map = self.inner.write().await;
        if let Some((url, _visited)) = map.iter().find(|(_, visited)| !**visited) {
            let url_clone = url.clone();
            map.remove(&url_clone);
            Some(url_clone)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::AbsoluteUrl;

    #[tokio::test]
    async fn test_insert_visited() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("page1"));

        // act
        let inserted = visited_urls.insert_visited(url.clone()).await;

        // assert
        assert!(inserted, "URL should be inserted");
        assert!(visited_urls.contains(&url).await, "URL should exist");
        assert!(
            visited_urls.is_visited(&url).await,
            "URL should be marked as visited"
        );
    }

    #[tokio::test]
    async fn test_insert_pending() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("page2"));

        // act
        let inserted = visited_urls.insert_pending(url.clone()).await;

        // assert
        assert!(inserted, "URL should be inserted");
        assert!(visited_urls.contains(&url).await, "URL should exist");
        assert!(
            !visited_urls.is_visited(&url).await,
            "URL should not be marked as visited"
        );
    }

    #[tokio::test]
    async fn test_insert_duplicate() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("page3"));

        // act
        visited_urls.insert_pending(url.clone()).await;
        let inserted_again = visited_urls.insert_visited(url.clone()).await;

        // assert
        assert!(!inserted_again, "Duplicate URL should not be inserted");
        assert_eq!(visited_urls.len().await, 1, "Should only have one URL");
    }

    #[tokio::test]
    async fn test_mark_visited() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("page4"));

        // act
        visited_urls.insert_pending(url.clone()).await;
        assert!(
            !visited_urls.is_visited(&url).await,
            "URL should not be visited initially"
        );

        let marked = visited_urls.mark_visited(url.clone()).await;

        // assert
        assert!(marked, "URL should be marked as visited");
        assert!(
            visited_urls.is_visited(&url).await,
            "URL should be visited after marking"
        );
    }

    #[tokio::test]
    async fn test_remove() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("page5"));

        // act
        visited_urls.insert_visited(url.clone()).await;
        let removed = visited_urls.remove(&url).await;

        // assert
        assert!(removed, "URL should be removed");
        assert!(
            !visited_urls.contains(&url).await,
            "URL should not exist after removal"
        );
        assert_eq!(visited_urls.len().await, 0, "Map should be empty");
    }

    #[tokio::test]
    async fn test_counts() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url1 = AbsoluteUrl::new("https://example.com", Some("page1"));
        let url2 = AbsoluteUrl::new("https://example.com", Some("page2"));
        let url3 = AbsoluteUrl::new("https://example.com", Some("page3"));

        // act
        visited_urls.insert_visited(url1).await;
        visited_urls.insert_pending(url2).await;
        visited_urls.insert_pending(url3).await;

        // assert
        assert_eq!(visited_urls.len().await, 3, "Total count should be 3");
        assert_eq!(
            visited_urls.visited_count().await,
            1,
            "Visited count should be 1"
        );
        assert_eq!(
            visited_urls.pending_count().await,
            2,
            "Pending count should be 2"
        );
    }

    #[tokio::test]
    async fn test_get_pending_urls() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url1 = AbsoluteUrl::new("https://example.com", Some("page1"));
        let url2 = AbsoluteUrl::new("https://example.com", Some("page2"));
        let url3 = AbsoluteUrl::new("https://example.com", Some("page3"));

        // act
        visited_urls.insert_visited(url1.clone()).await;
        visited_urls.insert_pending(url2.clone()).await;
        visited_urls.insert_pending(url3.clone()).await;

        let pending_urls = visited_urls.get_pending_urls().await;

        // assert
        assert_eq!(pending_urls.len(), 2, "Should have 2 pending URLs");
        assert!(pending_urls.contains(&url2), "Should contain url2");
        assert!(pending_urls.contains(&url3), "Should contain url3");
        assert!(
            !pending_urls.contains(&url1),
            "Should not contain visited url1"
        );
    }

    #[tokio::test]
    async fn test_pop_pending() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url1 = AbsoluteUrl::new("https://example.com", Some("page1"));
        let url2 = AbsoluteUrl::new("https://example.com", Some("page2"));

        // act
        visited_urls.insert_pending(url1.clone()).await;
        visited_urls.insert_visited(url2.clone()).await;

        let popped = visited_urls.pop_pending().await;

        // assert
        assert_eq!(popped, Some(url1), "Should pop the pending URL");
        assert_eq!(visited_urls.len().await, 1, "Should have 1 URL remaining");
        assert_eq!(
            visited_urls.pending_count().await,
            0,
            "Should have no pending URLs"
        );
    }

    #[tokio::test]
    async fn test_pop_pending_empty() {
        // arrange
        let visited_urls = VisitedUrls::new();

        // act
        let popped = visited_urls.pop_pending().await;

        // assert
        assert_eq!(popped, None, "Should return None when no pending URLs");
    }

    use tokio::task;

    #[tokio::test]
    async fn test_concurrent_inserts() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let visited_urls_clone1 = visited_urls.clone();
        let visited_urls_clone2 = visited_urls.clone();

        // act
        let task1 = task::spawn(async move {
            for i in 0..50 {
                let url = AbsoluteUrl::new("https://example.com", Some(&format!("page{}", i)));
                visited_urls_clone1.insert_pending(url).await;
            }
        });
        let task2 = task::spawn(async move {
            for i in 50..100 {
                let url = AbsoluteUrl::new("https://example.com", Some(&format!("page{}", i)));
                visited_urls_clone2.insert_pending(url).await;
            }
        });

        task1.await.unwrap();
        task2.await.unwrap();

        // assert
        assert_eq!(visited_urls.len().await, 100, "Should contain 100 URLs");
        assert_eq!(
            visited_urls.pending_count().await,
            100,
            "All URLs should be pending"
        );
    }

    #[tokio::test]
    async fn test_concurrent_duplicate_insertion_scenario() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url_c = AbsoluteUrl::new("https://example.com", Some("c"));

        // act
        let inserted_first = visited_urls.insert_pending(url_c.clone()).await;
        visited_urls.mark_visited(url_c.clone()).await;
        let inserted_second = visited_urls.insert_pending(url_c.clone()).await;

        // assert
        assert!(inserted_first, "First insertion should succeed");
        assert!(!inserted_second, "Second insertion should fail (duplicate)");
        assert_eq!(visited_urls.len().await, 1, "Should still have only 1 URL");
        assert!(
            visited_urls.is_visited(&url_c).await,
            "URL should remain visited after duplicate insertion"
        );
    }

    #[tokio::test]
    async fn test_concurrent_duplicate_with_different_states() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("test"));

        // act
        visited_urls.insert_pending(url.clone()).await;
        let inserted_as_visited = visited_urls.insert_visited(url.clone()).await;
        let still_pending = visited_urls.is_visited(&url).await;
        visited_urls.mark_visited(url.clone()).await;
        let inserted_as_pending = visited_urls.insert_pending(url.clone()).await;

        // assert
        assert!(!inserted_as_visited, "Duplicate insertion should fail");
        assert!(
            !still_pending,
            "Should still be pending after duplicate insert"
        );
        assert!(
            visited_urls.is_visited(&url).await,
            "Should be visited after marking"
        );
        assert!(!inserted_as_pending, "Duplicate insertion should fail");
        assert!(visited_urls.is_visited(&url).await, "Should remain visited");
    }

    #[tokio::test]
    async fn test_clear() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("test"));
        visited_urls.insert_pending(url.clone()).await;

        // act
        visited_urls.clear().await;
        let size = visited_urls.len().await;

        // assert
        assert_eq!(size, 0, "Length should be 0 after clear")
    }

    #[tokio::test]
    async fn test_get_visited_urls() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let first_url = AbsoluteUrl::new("https://example.com", Some("test"));
        let second_url = AbsoluteUrl::new("https://example.com/not_marked", Some("test"));

        visited_urls.insert_visited(first_url).await;
        visited_urls.insert_pending(second_url).await;

        // act
        let found = visited_urls.get_visited_urls().await;

        // assert
        assert_eq!(found.len(), 1, "Number of visited urls should be 1");
    }

    #[tokio::test]
    async fn test_concurrent_read_write() {
        // arrange
        let visited_urls = VisitedUrls::new();
        let url = AbsoluteUrl::new("https://example.com", Some("test"));
        visited_urls.insert_pending(url.clone()).await;

        let visited_urls_clone1 = visited_urls.clone();
        let visited_urls_clone2 = visited_urls.clone();
        let url_clone = url.clone();

        // act
        let writer = task::spawn(async move {
            for i in 0..10 {
                let url = AbsoluteUrl::new("https://example.com", Some(&format!("page{}", i)));
                visited_urls_clone1.insert_pending(url).await;
            }
        });
        let reader = task::spawn(async move {
            for _ in 0..10 {
                let _ = visited_urls_clone2.contains(&url_clone).await;
                let _ = visited_urls_clone2.len().await;
            }
        });

        writer.await.unwrap();
        reader.await.unwrap();

        // assert
        assert!(
            visited_urls.contains(&url).await,
            "Original URL should still exist"
        );
        assert_eq!(visited_urls.len().await, 11, "Should have 11 URLs total");
    }
}
