use async_trait::async_trait;
use chrono::Utc;
use clio::{ClioError, FetchStats, Fetcher, Item, Source};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test source that tracks when it was called
#[derive(Debug)]
struct TrackingSource {
    name: String,
    url: String,
    items: Vec<Item>,
    call_count: Arc<AtomicUsize>,
    delay_ms: u64,
    should_fail: bool,
}

#[async_trait]
impl Source for TrackingSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> &str {
        &self.url
    }

    async fn fetch(&self) -> Result<Vec<Item>, ClioError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        if self.should_fail {
            Err(ClioError::Network(format!(
                "Simulated failure for {}",
                self.name
            )))
        } else {
            Ok(self.items.clone())
        }
    }
}

fn create_test_item(id: &str, source_name: &str) -> Item {
    Item {
        id: id.to_string(),
        source_name: source_name.to_string(),
        title: format!("Test Article {id}"),
        link: format!("https://example.com/article/{id}"),
        summary: Some(format!("This is a test summary for article {id}")),
        pub_date: Some(Utc::now()),
    }
}

#[tokio::test]
async fn test_fetcher_with_real_network_mocks() {
    let mock_server = MockServer::start().await;

    // Set up mock RSS feed
    Mock::given(method("GET"))
        .and(path("/feed.rss"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <rss version="2.0">
                <channel>
                    <title>Test Feed</title>
                    <link>https://example.com</link>
                    <description>Test Description</description>
                    <item>
                        <title>Article 1</title>
                        <link>https://example.com/1</link>
                        <description>Description 1</description>
                        <pubDate>Wed, 01 Jan 2025 12:00:00 GMT</pubDate>
                    </item>
                </channel>
            </rss>"#,
        ))
        .mount(&mock_server)
        .await;

    // Create RSS source
    use clio::source::rss::RssSource;
    let source = Arc::new(RssSource::new(
        "Mock Feed".to_string(),
        format!("{}/feed.rss", &mock_server.uri()),
    ));

    let fetcher = Fetcher::new();
    let (items, stats) = fetcher.fetch_all(vec![source]).await;

    assert_eq!(stats.successful_sources, 1);
    assert_eq!(stats.failed_sources, 0);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Article 1");
}

#[tokio::test]
async fn test_parallel_fetch_with_varying_delays() {
    let sources: Vec<Arc<dyn Source>> = (0..10)
        .map(|i| {
            Arc::new(TrackingSource {
                name: format!("Source{i}"),
                url: format!("https://source{i}.com/feed"),
                items: vec![create_test_item(&format!("{i}"), &format!("Source{i}"))],
                call_count: Arc::new(AtomicUsize::new(0)),
                delay_ms: ((i % 3) + 1) as u64 * 50, // Varying delays: 50, 100, 150ms
                should_fail: false,
            }) as Arc<dyn Source>
        })
        .collect();

    let fetcher = Fetcher::new();
    let start = tokio::time::Instant::now();
    let (items, stats) = fetcher.fetch_all(sources).await;
    let elapsed = start.elapsed();

    // Should be much faster than sequential (which would be ~1000ms total)
    assert!(
        elapsed < Duration::from_millis(500),
        "Parallel fetching took too long: {elapsed:?}"
    );
    assert_eq!(items.len(), 10);
    assert_eq!(stats.successful_sources, 10);
    assert_eq!(stats.total_items, 10);
}

#[tokio::test]
async fn test_mixed_success_and_failure() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(TrackingSource {
            name: "Success1".to_string(),
            url: "https://success1.com/feed".to_string(),
            items: vec![
                create_test_item("1", "Success1"),
                create_test_item("2", "Success1"),
            ],
            call_count: call_count.clone(),
            delay_ms: 0,
            should_fail: false,
        }),
        Arc::new(TrackingSource {
            name: "Failure1".to_string(),
            url: "https://failure1.com/feed".to_string(),
            items: vec![],
            call_count: call_count.clone(),
            delay_ms: 0,
            should_fail: true,
        }),
        Arc::new(TrackingSource {
            name: "Success2".to_string(),
            url: "https://success2.com/feed".to_string(),
            items: vec![create_test_item("3", "Success2")],
            call_count: call_count.clone(),
            delay_ms: 0,
            should_fail: false,
        }),
        Arc::new(TrackingSource {
            name: "Failure2".to_string(),
            url: "https://failure2.com/feed".to_string(),
            items: vec![],
            call_count: call_count.clone(),
            delay_ms: 0,
            should_fail: true,
        }),
    ];

    let fetcher = Fetcher::new();
    let (items, stats) = fetcher.fetch_all(sources).await;

    assert_eq!(call_count.load(Ordering::SeqCst), 4); // All sources were called
    assert_eq!(items.len(), 3);
    assert_eq!(stats.successful_sources, 2);
    assert_eq!(stats.failed_sources, 2);
    assert_eq!(stats.errors.len(), 2);

    // Check error messages
    let error_sources: Vec<String> = stats.errors.iter().map(|(s, _)| s.clone()).collect();
    assert!(error_sources.contains(&"Failure1".to_string()));
    assert!(error_sources.contains(&"Failure2".to_string()));
}

#[tokio::test]
async fn test_timeout_handling() {
    let sources: Vec<Arc<dyn Source>> = vec![
        Arc::new(TrackingSource {
            name: "FastSource".to_string(),
            url: "https://fast.com/feed".to_string(),
            items: vec![create_test_item("1", "FastSource")],
            call_count: Arc::new(AtomicUsize::new(0)),
            delay_ms: 50,
            should_fail: false,
        }),
        Arc::new(TrackingSource {
            name: "SlowSource".to_string(),
            url: "https://slow.com/feed".to_string(),
            items: vec![create_test_item("2", "SlowSource")],
            call_count: Arc::new(AtomicUsize::new(0)),
            delay_ms: 3000, // Will timeout with 1s timeout
            should_fail: false,
        }),
    ];

    let fetcher = Fetcher::with_timeout(1); // 1 second timeout
    let (items, stats) = fetcher.fetch_all(sources).await;

    assert_eq!(items.len(), 1); // Only fast source completes
    assert_eq!(stats.successful_sources, 1);
    assert_eq!(stats.failed_sources, 1);
    assert_eq!(stats.errors[0].0, "SlowSource");
    assert!(stats.errors[0].1.contains("timed out"));
}

#[tokio::test]
async fn test_empty_source_list() {
    let fetcher = Fetcher::new();
    let (items, stats) = fetcher.fetch_all(vec![]).await;

    assert_eq!(items.len(), 0);
    assert_eq!(stats.num_sources, 0);
    assert_eq!(stats.successful_sources, 0);
    assert_eq!(stats.failed_sources, 0);
    assert_eq!(stats.errors.len(), 0);
}

#[tokio::test]
async fn test_large_batch_fetching() {
    // Create 50 sources with varying characteristics
    let sources: Vec<Arc<dyn Source>> = (0..50)
        .map(|i| {
            let should_fail = i % 10 == 0; // 10% failure rate
            let delay = if i % 5 == 0 { 100 } else { 10 }; // Some slow sources

            Arc::new(TrackingSource {
                name: format!("Source{i:02}"),
                url: format!("https://source{i}.com/feed"),
                items: if should_fail {
                    vec![]
                } else {
                    vec![
                        create_test_item(&format!("{i}a"), &format!("Source{i:02}")),
                        create_test_item(&format!("{i}b"), &format!("Source{i:02}")),
                    ]
                },
                call_count: Arc::new(AtomicUsize::new(0)),
                delay_ms: delay,
                should_fail,
            }) as Arc<dyn Source>
        })
        .collect();

    let fetcher = Fetcher::new();
    let start = tokio::time::Instant::now();
    let (items, stats) = fetcher.fetch_all(sources).await;
    let elapsed = start.elapsed();

    // Should complete quickly even with 50 sources
    assert!(
        elapsed < Duration::from_secs(2),
        "Large batch took too long: {elapsed:?}"
    );

    assert_eq!(stats.num_sources, 50);
    assert_eq!(stats.successful_sources, 45); // 90% success rate
    assert_eq!(stats.failed_sources, 5); // 10% failure rate
    assert_eq!(items.len(), 90); // 45 sources * 2 items each
}

#[tokio::test]
async fn test_fetch_result_processing() {
    let mut stats = FetchStats::new(3);

    stats.process_result(&clio::FetchResult::Success {
        source_name: "Source1".to_string(),
        items: vec![
            create_test_item("1", "Source1"),
            create_test_item("2", "Source1"),
        ],
    });

    stats.process_result(&clio::FetchResult::Error {
        source_name: "Source2".to_string(),
        error: "Network error".to_string(),
    });

    stats.process_result(&clio::FetchResult::Success {
        source_name: "Source3".to_string(),
        items: vec![create_test_item("3", "Source3")],
    });

    assert_eq!(stats.num_sources, 3);
    assert_eq!(stats.successful_sources, 2);
    assert_eq!(stats.failed_sources, 1);
    assert_eq!(stats.total_items, 3);
    assert_eq!(stats.errors.len(), 1);
    assert_eq!(stats.errors[0].0, "Source2");
}

#[tokio::test]
async fn test_concurrent_access_safety() {
    let shared_counter = Arc::new(AtomicUsize::new(0));

    let sources: Vec<Arc<dyn Source>> = (0..20)
        .map(|i| {
            Arc::new(TrackingSource {
                name: format!("Source{i}"),
                url: format!("https://source{i}.com/feed"),
                items: vec![create_test_item(&format!("{i}"), &format!("Source{i}"))],
                call_count: shared_counter.clone(),
                delay_ms: 10,
                should_fail: false,
            }) as Arc<dyn Source>
        })
        .collect();

    let fetcher = Fetcher::new();
    let (items, stats) = fetcher.fetch_all(sources).await;

    assert_eq!(shared_counter.load(Ordering::SeqCst), 20);
    assert_eq!(items.len(), 20);
    assert_eq!(stats.successful_sources, 20);
}

#[tokio::test]
async fn test_stats_display_summary() {
    let mut stats = FetchStats::new(3);
    stats.successful_sources = 2;
    stats.failed_sources = 1;
    stats.total_items = 5;
    stats.errors = vec![("BadSource".to_string(), "Connection failed".to_string())];

    // This test verifies the display_summary method runs without panic
    stats.display_summary();
}
