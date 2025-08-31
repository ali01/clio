use async_trait::async_trait;
use chrono::Utc;
use clio::{ClioError, Fetcher, Item, Source};
use criterion::{Criterion, criterion_group, criterion_main};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
struct BenchmarkSource {
    name: String,
    url: String,
    items: Vec<Item>,
    delay_ms: u64,
    should_fail: bool,
}

#[async_trait]
impl Source for BenchmarkSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> &str {
        &self.url
    }

    async fn fetch(&self) -> Result<Vec<Item>, ClioError> {
        if self.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
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
        title: format!("Benchmark Article {id}"),
        link: format!("https://example.com/article/{id}"),
        summary: Some(format!("This is a benchmark summary for article {id}")),
        pub_date: Some(Utc::now()),
    }
}

fn create_100_sources() -> Vec<Arc<dyn Source>> {
    (0..100)
        .map(|i| {
            // 5% failure rate, varying delays
            let should_fail = i % 20 == 0;
            let delay = if i % 10 == 0 {
                50 // 10% slow sources
            } else if i % 5 == 0 {
                20 // 20% medium sources
            } else {
                5 // 70% fast sources
            };

            let items = if should_fail {
                vec![]
            } else {
                (0..3)
                    .map(|j| create_test_item(&format!("{i}-{j}"), &format!("Source{i:03}")))
                    .collect()
            };

            Arc::new(BenchmarkSource {
                name: format!("Source{i:03}"),
                url: format!("https://source{i}.example.com/feed"),
                items,
                delay_ms: delay,
                should_fail,
            }) as Arc<dyn Source>
        })
        .collect()
}

fn benchmark_fetch_100_sources(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fetch_100_sources", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let sources = create_100_sources();
                let fetcher = Fetcher::new();
                let (items, stats) = fetcher.fetch_all(sources).await;

                // Verify results for correctness
                assert!(stats.num_sources == 100);
                assert!(stats.successful_sources >= 90); // At least 90% success
                assert!(items.len() >= 270); // At least 90 sources * 3 items

                (items, stats)
            })
        });
    });
}

fn benchmark_fetch_10_sources(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fetch_10_sources", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let sources = create_100_sources().into_iter().take(10).collect();
                let fetcher = Fetcher::new();
                fetcher.fetch_all(sources).await
            })
        });
    });
}

fn benchmark_fetch_single_source(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("fetch_single_source", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let source = Arc::new(BenchmarkSource {
                    name: "Single".to_string(),
                    url: "https://single.example.com/feed".to_string(),
                    items: vec![
                        create_test_item("1", "Single"),
                        create_test_item("2", "Single"),
                        create_test_item("3", "Single"),
                    ],
                    delay_ms: 10,
                    should_fail: false,
                });

                let fetcher = Fetcher::new();
                fetcher.fetch_one(source).await
            })
        });
    });
}

fn benchmark_memory_usage_100_sources(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("memory_100_sources", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let sources = create_100_sources();
                let fetcher = Fetcher::new();

                // Measure memory by forcing all allocations
                let (items, stats) = fetcher.fetch_all(sources).await;

                // Force retention of all data
                let total_memory_estimate =
                    items.len() * std::mem::size_of::<Item>() + stats.errors.len() * 100; // Estimate for error strings

                // Verify memory usage is reasonable
                assert!(total_memory_estimate < 10_000_000); // Less than 10MB

                (items, stats)
            })
        });
    });
}

fn benchmark_timeout_behavior(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("timeout_behavior", |b| {
        b.iter(|| {
            runtime.block_on(async {
                // Mix of fast and slow sources to test timeout
                let sources: Vec<Arc<dyn Source>> = vec![
                    Arc::new(BenchmarkSource {
                        name: "Fast".to_string(),
                        url: "https://fast.example.com/feed".to_string(),
                        items: vec![create_test_item("1", "Fast")],
                        delay_ms: 10,
                        should_fail: false,
                    }),
                    Arc::new(BenchmarkSource {
                        name: "Slow".to_string(),
                        url: "https://slow.example.com/feed".to_string(),
                        items: vec![create_test_item("2", "Slow")],
                        delay_ms: 11000, // Will timeout
                        should_fail: false,
                    }),
                ];

                let fetcher = Fetcher::with_timeout(1); // 1 second timeout
                fetcher.fetch_all(sources).await
            })
        });
    });
}

criterion_group!(
    benches,
    benchmark_fetch_100_sources,
    benchmark_fetch_10_sources,
    benchmark_fetch_single_source,
    benchmark_memory_usage_100_sources,
    benchmark_timeout_behavior
);

criterion_main!(benches);
