use crate::error::ClioError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::fmt::Debug;

pub mod rss;

/// Represents a single content item from any source
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Unique identifier for the session
    pub id: String,
    /// Name from configuration
    pub source_name: String,
    /// Article title
    pub title: String,
    /// URL to the article
    pub link: String,
    /// Article summary/description
    pub summary: Option<String>,
    /// Publication date
    pub pub_date: Option<DateTime<Utc>>,
}

/// Trait for all content sources
#[async_trait]
pub trait Source: Send + Sync + Debug {
    /// Get the name of this source
    fn name(&self) -> &str;

    /// Get the URL of this source
    fn url(&self) -> &str;
    
    /// Pull all items from this source
    async fn pull(&self) -> Result<Vec<Item>, ClioError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock source for testing
    #[derive(Debug)]
    struct MockSource {
        name: String,
        url: String,
        items: Vec<Item>,
        should_fail: bool,
    }

    #[async_trait]
    impl Source for MockSource {
        async fn pull(&self) -> Result<Vec<Item>, ClioError> {
            if self.should_fail {
                Err(ClioError::Network("Mock network error".to_string()))
            } else {
                Ok(self.items.clone())
            }
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn url(&self) -> &str {
            &self.url
        }
    }

    #[tokio::test]
    async fn test_mock_source_success() {
        let items = vec![Item {
            id: "1".to_string(),
            source_name: "Test Source".to_string(),
            title: "Test Article".to_string(),
            link: "https://example.com/article".to_string(),
            summary: Some("Test summary".to_string()),
            pub_date: Some(Utc::now()),
        }];

        let source = MockSource {
            name: "Test Source".to_string(),
            url: "https://example.com/feed".to_string(),
            items: items.clone(),
            should_fail: false,
        };

        let result = source.pull().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), items);
        assert_eq!(source.name(), "Test Source");
        assert_eq!(source.url(), "https://example.com/feed");
    }

    #[tokio::test]
    async fn test_mock_source_failure() {
        let source = MockSource {
            name: "Test Source".to_string(),
            url: "https://example.com/feed".to_string(),
            items: vec![],
            should_fail: true,
        };

        let result = source.pull().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClioError::Network(_)));
    }

    #[test]
    fn test_item_equality() {
        let item1 = Item {
            id: "1".to_string(),
            source_name: "Source".to_string(),
            title: "Title".to_string(),
            link: "https://example.com".to_string(),
            summary: None,
            pub_date: None,
        };

        let item2 = item1.clone();
        assert_eq!(item1, item2);

        let item3 = Item {
            id: "2".to_string(),
            ..item1.clone()
        };
        assert_ne!(item1, item3);
    }
}
