use crate::{ClioError, Item, Source};
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Fetcher handles parallel content fetching from multiple sources
pub struct Fetcher {
    timeout_duration: Duration,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    /// Create a new fetcher with default timeout
    pub fn new() -> Self {
        Self {
            timeout_duration: Duration::from_secs(10),
        }
    }

    /// Create a fetcher with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            timeout_duration: Duration::from_secs(timeout_secs),
        }
    }

    /// Fetch content from all sources in parallel
    pub async fn fetch_all(&self, sources: Vec<Arc<dyn Source>>) -> (Vec<Item>, FetchStats) {
        let num_sources = sources.len();
        let mut stats = FetchStats::new(num_sources);

        if sources.is_empty() {
            return (Vec::new(), stats);
        }

        // Show initial progress
        println!("Fetching content from {num_sources} sources...");

        // Create concurrent fetch async_tasks
        let async_tasks: Vec<_> = sources
            .into_iter()
            .enumerate()
            .map(|(index, source)| {
                let fetcher = Self::with_timeout(self.timeout_duration.as_secs());

                tokio::spawn(async move {
                    // Show progress for this source
                    let source_name = source.name().to_string();
                    println!("  [{}/{}] Fetching {}", index + 1, num_sources, source_name);

                    // Use fetch_one to handle timeout logic
                    match fetcher.fetch_one(source).await {
                        Ok(items) => FetchResult::Success {
                            source_name: source_name.clone(),
                            items,
                        },
                        Err(e) => FetchResult::Error {
                            source_name: source_name.clone(),
                            error: e.to_string(),
                        },
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete and collect results
        let results = join_all(async_tasks).await;

        // Process all results
        let mut feed_items = Vec::new();
        for result in results {
            match result {
                Ok(fetch_result) => {
                    if let FetchResult::Success { ref items, .. } = fetch_result {
                        feed_items.extend(items.clone());
                    }
                    stats.process_result(&fetch_result);
                }
                Err(e) => {
                    stats.failed_sources += 1;
                    stats
                        .errors
                        .push(("Unknown".to_string(), format!("Task failed: {e}")));
                }
            }
        }

        println!(); // Empty line after progress
        stats.display_summary();

        (feed_items, stats)
    }

    /// Fetch from a single source with timeout
    pub async fn fetch_one(&self, source: Arc<dyn Source>) -> Result<Vec<Item>, ClioError> {
        timeout(self.timeout_duration, source.fetch())
            .await
            .map_err(|_| {
                ClioError::Network(format!(
                    "Request to {} timed out after {:?}",
                    source.name(),
                    self.timeout_duration
                ))
            })?
    }
}

/// Result from fetching a single source
#[derive(Debug, Clone)]
pub enum FetchResult {
    Success {
        source_name: String,
        items: Vec<Item>,
    },
    Error {
        source_name: String,
        error: String,
    },
}

/// Statistics from a fetch operation
#[derive(Debug, Clone, Default)]
pub struct FetchStats {
    pub num_sources: usize,
    pub successful_sources: usize,
    pub failed_sources: usize,
    pub total_items: usize,
    pub errors: Vec<(String, String)>, // (source_name, error_message)
}

impl FetchStats {
    /// Create new fetch statistics
    pub fn new(num_sources: usize) -> Self {
        Self {
            num_sources,
            successful_sources: 0,
            failed_sources: 0,
            total_items: 0,
            errors: Vec::new(),
        }
    }

    /// Process a fetch result and update statistics
    pub fn process_result(&mut self, result: &FetchResult) {
        match result {
            FetchResult::Success { items, .. } => {
                self.successful_sources += 1;
                self.total_items += items.len();
            }
            FetchResult::Error { source_name, error } => {
                self.failed_sources += 1;
                self.errors.push((source_name.clone(), error.clone()));
            }
        }
    }

    /// Display summary of fetch operation
    pub fn display_summary(&self) {
        println!(
            "Fetched {} items from {} of {} sources",
            self.total_items, self.successful_sources, self.num_sources
        );

        if !self.errors.is_empty() {
            eprintln!("\nFailed sources:");
            for (source, error) in &self.errors {
                eprintln!("  - {source}: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Source;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::atomic::AtomicUsize;
    use tokio::time::sleep;

    #[derive(Debug)]
    struct MockSource {
        name: String,
        url: String,
        items: Vec<Item>,
        delay_ms: u64,
        should_fail: bool,
    }

    #[async_trait]
    impl Source for MockSource {
        fn name(&self) -> &str {
            &self.name
        }

        fn url(&self) -> &str {
            &self.url
        }

        async fn fetch(&self) -> Result<Vec<Item>, ClioError> {
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if self.should_fail {
                Err(ClioError::Network("Mock network error".to_string()))
            } else {
                Ok(self.items.clone())
            }
        }
    }

    fn create_test_item(id: &str, source_name: &str) -> Item {
        Item {
            id: id.to_string(),
            source_name: source_name.to_string(),
            title: format!("Article {id}"),
            link: format!("https://example.com/{id}"),
            summary: Some(format!("Summary for article {id}")),
            pub_date: Some(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_fetch_single_source() {
        let source = Arc::new(MockSource {
            name: "Test Source".to_string(),
            url: "https://example.com/feed".to_string(),
            items: vec![
                create_test_item("1", "Test Source"),
                create_test_item("2", "Test Source"),
            ],
            delay_ms: 0,
            should_fail: false,
        });

        let fetcher = Fetcher::new();
        let result = fetcher.fetch_one(source).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_fetch_multiple_sources_parallel() {
        let sources: Vec<Arc<dyn Source>> = vec![
            Arc::new(MockSource {
                name: "Source1".to_string(),
                url: "https://example1.com/feed".to_string(),
                items: vec![create_test_item("1", "Source1")],
                delay_ms: 100,
                should_fail: false,
            }),
            Arc::new(MockSource {
                name: "Source2".to_string(),
                url: "https://example2.com/feed".to_string(),
                items: vec![create_test_item("2", "Source2")],
                delay_ms: 100,
                should_fail: false,
            }),
            Arc::new(MockSource {
                name: "Source3".to_string(),
                url: "https://example3.com/feed".to_string(),
                items: vec![create_test_item("3", "Source3")],
                delay_ms: 100,
                should_fail: false,
            }),
        ];

        let fetcher = Fetcher::new();
        let start = tokio::time::Instant::now();
        let (items, stats) = fetcher.fetch_all(sources).await;
        let elapsed = start.elapsed();

        // Should complete in ~100ms (parallel), not 300ms (sequential)
        assert!(elapsed < Duration::from_millis(200));
        assert_eq!(items.len(), 3);
        assert_eq!(stats.successful_sources, 3);
        assert_eq!(stats.failed_sources, 0);
        assert_eq!(stats.total_items, 3);
    }

    #[tokio::test]
    async fn test_fetch_with_failures() {
        let sources: Vec<Arc<dyn Source>> = vec![
            Arc::new(MockSource {
                name: "GoodSource".to_string(),
                url: "https://good.com/feed".to_string(),
                items: vec![create_test_item("1", "GoodSource")],
                delay_ms: 0,
                should_fail: false,
            }),
            Arc::new(MockSource {
                name: "BadSource".to_string(),
                url: "https://bad.com/feed".to_string(),
                items: vec![],
                delay_ms: 0,
                should_fail: true,
            }),
        ];

        let fetcher = Fetcher::new();
        let (items, stats) = fetcher.fetch_all(sources).await;

        assert_eq!(items.len(), 1);
        assert_eq!(stats.successful_sources, 1);
        assert_eq!(stats.failed_sources, 1);
        assert_eq!(stats.errors.len(), 1);
        assert_eq!(stats.errors[0].0, "BadSource");
    }

    #[tokio::test]
    async fn test_fetch_with_timeout() {
        let source = Arc::new(MockSource {
            name: "SlowSource".to_string(),
            url: "https://slow.com/feed".to_string(),
            items: vec![],
            delay_ms: 5000, // 5 seconds
            should_fail: false,
        });

        let fetcher = Fetcher::with_timeout(1); // 1 second timeout
        let result = fetcher.fetch_one(source).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClioError::Network(_)));
    }

    #[tokio::test]
    async fn test_fetch_empty_sources() {
        let sources: Vec<Arc<dyn Source>> = vec![];
        let fetcher = Fetcher::new();
        let (items, stats) = fetcher.fetch_all(sources).await;

        assert_eq!(items.len(), 0);
        assert_eq!(stats.num_sources, 0);
        assert_eq!(stats.successful_sources, 0);
        assert_eq!(stats.failed_sources, 0);
    }

    #[tokio::test]
    async fn test_concurrent_fetching_order() {
        // Track order of completion
        let _counter = Arc::new(AtomicUsize::new(0));

        let make_source = |name: &str, delay: u64| -> Arc<dyn Source> {
            Arc::new(MockSource {
                name: name.to_string(),
                url: format!("https://{}.com/feed", name.to_lowercase()),
                items: vec![create_test_item(name, name)],
                delay_ms: delay,
                should_fail: false,
            })
        };

        let sources: Vec<Arc<dyn Source>> = vec![
            make_source("Fast", 10),
            make_source("Medium", 50),
            make_source("Slow", 100),
        ];

        let fetcher = Fetcher::new();
        let (items, stats) = fetcher.fetch_all(sources).await;

        // All should complete regardless of order
        assert_eq!(items.len(), 3);
        assert_eq!(stats.successful_sources, 3);
    }

    #[tokio::test]
    async fn test_fetch_stats_calculation() {
        let sources: Vec<Arc<dyn Source>> = vec![
            Arc::new(MockSource {
                name: "Source1".to_string(),
                url: "https://1.com/feed".to_string(),
                items: vec![
                    create_test_item("1", "Source1"),
                    create_test_item("2", "Source1"),
                ],
                delay_ms: 0,
                should_fail: false,
            }),
            Arc::new(MockSource {
                name: "Source2".to_string(),
                url: "https://2.com/feed".to_string(),
                items: vec![
                    create_test_item("3", "Source2"),
                    create_test_item("4", "Source2"),
                    create_test_item("5", "Source2"),
                ],
                delay_ms: 0,
                should_fail: false,
            }),
            Arc::new(MockSource {
                name: "Source3".to_string(),
                url: "https://3.com/feed".to_string(),
                items: vec![],
                delay_ms: 0,
                should_fail: true,
            }),
        ];

        let fetcher = Fetcher::new();
        let (items, stats) = fetcher.fetch_all(sources).await;

        assert_eq!(stats.num_sources, 3);
        assert_eq!(stats.successful_sources, 2);
        assert_eq!(stats.failed_sources, 1);
        assert_eq!(stats.total_items, 5);
        assert_eq!(items.len(), 5);
    }

    #[tokio::test]
    async fn test_fetch_result_enum() {
        let success = FetchResult::Success {
            source_name: "Test".to_string(),
            items: vec![create_test_item("1", "Test")],
        };

        let error = FetchResult::Error {
            source_name: "Failed".to_string(),
            error: "Network error".to_string(),
        };

        let mut stats = FetchStats::new(2);
        stats.process_result(&success);
        stats.process_result(&error);

        assert_eq!(stats.successful_sources, 1);
        assert_eq!(stats.failed_sources, 1);
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.errors.len(), 1);
    }

    #[test]
    fn test_fetcher_default() {
        let fetcher = Fetcher::default();
        assert_eq!(fetcher.timeout_duration, Duration::from_secs(10));
    }

    #[test]
    fn test_fetcher_with_custom_timeout() {
        let fetcher = Fetcher::with_timeout(30);
        assert_eq!(fetcher.timeout_duration, Duration::from_secs(30));
    }
}
