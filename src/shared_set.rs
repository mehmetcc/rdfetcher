use std::borrow::{Borrow, Cow};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SharedSet<'a> {
    inner: Arc<RwLock<HashSet<Cow<'a, str>>>>,
}

impl<'a> SharedSet<'a> {
    pub fn new() -> Self {
        SharedSet {
            inner: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn insert<Q>(&self, value: Q) -> bool
    where
        Q: Into<Cow<'a, str>>,
    {
        let mut set = self.inner.write().await;
        set.insert(value.into())
    }

    pub async fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: Hash + Eq + ?Sized,
        Cow<'a, str>: Borrow<Q>,
    {
        let set = self.inner.read().await;
        set.contains(value)
    }

    pub async fn remove<Q>(&self, value: &Q) -> bool
    where
        Q: Hash + Eq + ?Sized,
        Cow<'a, str>: Borrow<Q>,
    {
        let mut set = self.inner.write().await;
        set.remove(value)
    }

    pub async fn len(&self) -> usize {
        let set = self.inner.read().await;
        set.len()
    }

    pub async fn clear(&self) {
        let mut set = self.inner.write().await;
        set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert() {
        // arrange
        let set = SharedSet::new();

        // act
        set.insert("anan").await;
        set.insert("baban".to_string()).await;

        // assert
        assert!(
            set.contains("anan").await,
            "Set should contain inserted value"
        );
        assert!(
            set.contains("baban").await,
            "Set should contain inserted value"
        );
        assert!(
            !set.contains("i don't exist biatch").await,
            "Set should not contain non-inserted value"
        )
    }

    #[tokio::test]
    async fn test_insert_should_not_insert_more_than_once() {
        // arrange
        let set = SharedSet::new();

        // act
        set.insert("anan").await;
        set.insert("baban".to_string()).await;
        set.insert("anan").await;

        // assert
        assert_eq!(
            set.len().await,
            2,
            "Set should not insert the same element more than once"
        )
    }

    #[tokio::test]
    async fn test_remove() {
        // arrange
        let set = SharedSet::new();

        // act
        set.insert("anan").await;
        set.remove("anan").await;

        set.insert("baban".to_string()).await;
        set.remove("baban").await;

        // assert
        assert_eq!(
            set.len().await,
            0,
            "Set should remove the specified element"
        )
    }

    #[tokio::test]
    async fn test_clear() {
        // arrange
        let set = SharedSet::new();

        // act
        set.insert("one").await;
        set.insert("two").await;
        set.insert("three").await;

        // Clear the set
        set.clear().await;
        assert_eq!(set.len().await, 0, "Set should be empty after clear");
    }

    use tokio::task;

    #[tokio::test]
    async fn test_concurrent_inserts() {
        // arrange
        let set = SharedSet::new();
        let set_clone1 = set.clone();
        let set_clone2 = set.clone();

        // act
        let task1 = task::spawn(async move {
            for i in 0..50 {
                set_clone1.insert(i.to_string()).await;
            }
        });
        let task2 = task::spawn(async move {
            for i in 50..100 {
                set_clone2.insert(i.to_string()).await;
            }
        });

        task1.await.unwrap();
        task2.await.unwrap();

        // assert
        assert_eq!(set.len().await, 100, "Set should contain 100 items");
        for i in 0..100 {
            let current = i.to_string();
            let reference = current.as_str();
            assert!(
                set.contains(reference).await,
                "Set should contain value {}",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_read_write() {
        // arrange
        let set = SharedSet::new();
        set.insert("some bitch").await;
        let set_clone1 = set.clone();
        let set_clone2 = set.clone();

        // act
        let writer = task::spawn(async move {
            for i in 0..10 {
                set_clone1.insert("some other bitch").await;
                set_clone1.remove("some other bitch").await;
            }
        });
        let reader = task::spawn(async move {
            for _ in 0..10 {
                let _ = set_clone2.contains("some bitch").await;
                let _ = set_clone2.len().await;
            }
        });
        writer.await.unwrap();
        reader.await.unwrap();

        // assert
        assert!(
            set.contains("some bitch").await,
            "Set should still contain initial value"
        );
    }
}
