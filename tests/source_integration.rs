use clio::source::Source;
use clio::source::rss::RssSource;
use std::fs;
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|_| panic!("Failed to read fixture: {name}"))
}

#[tokio::test]
async fn test_parse_rss_detailed() {
    let mock_server = MockServer::start().await;
    let rss_content = read_fixture("sample_rss_detailed.xml");

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(rss_content))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Tech News".to_string(),
        format!("{}/feed.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok(), "Failed to pull RSS feed: {result:?}");

    let items = result.unwrap();
    assert_eq!(items.len(), 3);

    // Verify first item
    let first = &items[0];
    assert_eq!(first.title, "Breaking: New Rust Framework Released");
    assert_eq!(first.link, "https://technews.example.com/rust-framework");
    assert!(first.summary.is_some());
    assert!(first.summary.as_ref().unwrap().contains("revolutionary"));
    assert!(
        first.pub_date.is_some(),
        "First item should have a pub_date"
    );

    // Verify all items have the correct source name
    for item in &items {
        assert_eq!(item.source_name, "Tech News");
        assert!(!item.id.is_empty());
    }
}

#[tokio::test]
async fn test_parse_atom_detailed() {
    let mock_server = MockServer::start().await;
    let atom_content = read_fixture("sample_atom_detailed.xml");

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(atom_content))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Dev Blog".to_string(),
        format!("{}/feed.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok(), "Failed to pull Atom feed: {result:?}");

    let items = result.unwrap();
    assert_eq!(items.len(), 3);

    // Verify first item
    let first = &items[0];
    assert_eq!(first.title, "Understanding Async Rust");
    assert_eq!(first.link, "https://devblog.example.com/async-rust");
    assert!(first.summary.is_some());
    assert!(first.pub_date.is_some());

    // Verify second item (has only summary)
    let second = &items[1];
    assert_eq!(second.title, "Building CLI Tools with Clap");
    assert!(second.summary.is_some());
}

#[tokio::test]
async fn test_parse_rss_with_entities() {
    let mock_server = MockServer::start().await;
    let rss_content = read_fixture("rss_with_entities.xml");

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(rss_content))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Entity Test".to_string(),
        format!("{}/feed.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok(), "Failed to pull with entities: {result:?}");

    let items = result.unwrap();
    assert_eq!(items.len(), 3);

    // Test HTML entity decoding
    let first = &items[0];
    assert_eq!(first.title, "Title with & ampersand <tags> and \"quotes\"");
    assert!(first.summary.as_ref().unwrap().contains("'apostrophes'"));
    assert!(first.summary.as_ref().unwrap().contains("©")); // Numeric entity decoded
    assert!(first.summary.as_ref().unwrap().contains("®"));
    assert!(first.summary.as_ref().unwrap().contains("™"));

    // Test Unicode preservation
    let second = &items[1];
    assert!(second.title.contains("café"));
    assert!(second.title.contains("日本語"));
    assert!(second.summary.as_ref().unwrap().contains("🚀"));
    assert!(second.summary.as_ref().unwrap().contains("中文"));
}

#[tokio::test]
async fn test_parse_various_date_formats() {
    let mock_server = MockServer::start().await;
    let rss_content = read_fixture("rss_date_formats.xml");

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(rss_content))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Date Test".to_string(),
        format!("{}/feed.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok());

    let items = result.unwrap();

    // Should have 8 items total
    assert!(
        items.len() >= 6,
        "Expected at least 6 items, got {}",
        items.len()
    );

    // Count items with valid dates
    let items_with_dates: Vec<_> = items
        .iter()
        .filter(|item| item.pub_date.is_some())
        .collect();

    // At least some date formats should parse successfully
    assert!(
        items_with_dates.len() >= 4,
        "Expected at least 4 items with valid dates, got {}",
        items_with_dates.len()
    );

    // Verify items without dates are still included
    let items_without_dates: Vec<_> = items
        .iter()
        .filter(|item| item.pub_date.is_none())
        .collect();

    assert!(
        !items_without_dates.is_empty(),
        "Should have items without dates"
    );
}

#[tokio::test]
async fn test_malformed_feed_error() {
    let mock_server = MockServer::start().await;
    let malformed_content = read_fixture("malformed_rss.xml");

    Mock::given(method("GET"))
        .and(path("/feed.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(malformed_content))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Malformed Test".to_string(),
        format!("{}/feed.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_err(), "Should fail to parse malformed XML");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("parse") || error.to_string().contains("Parse"),
        "Error should mention parsing: {error}"
    );
}

#[tokio::test]
async fn test_http_404_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/missing.xml"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "404 Test".to_string(),
        format!("{}/missing.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("404") || error.to_string().contains("HTTP"),
        "Error should mention HTTP status: {error}"
    );
}

#[tokio::test]
async fn test_http_500_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error.xml"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "500 Test".to_string(),
        format!("{}/error.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("500") || error.to_string().contains("HTTP"),
        "Error should mention HTTP status: {error}"
    );
}

#[tokio::test]
async fn test_empty_feed() {
    let mock_server = MockServer::start().await;
    let empty_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
    <link>https://example.com</link>
    <description>No items</description>
  </channel>
</rss>"#;

    Mock::given(method("GET"))
        .and(path("/empty.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(empty_rss))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Empty Test".to_string(),
        format!("{}/empty.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_items_with_missing_fields() {
    let mock_server = MockServer::start().await;
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <item>
      <title></title>
      <link>https://example.com/empty-title</link>
      <description>Should be skipped - empty title</description>
    </item>
    <item>
      <title>   </title>
      <link>https://example.com/whitespace-title</link>
      <description>Should be skipped - whitespace title</description>
    </item>
    <item>
      <title>No Link Item</title>
      <description>Should be skipped - no link</description>
    </item>
    <item>
      <title>Valid Item</title>
      <link>https://example.com/valid</link>
    </item>
    <item>
      <title>Another Valid Item</title>
      <link>https://example.com/valid2</link>
      <description>This one has a description</description>
    </item>
  </channel>
</rss>"#;

    Mock::given(method("GET"))
        .and(path("/partial.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(rss))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Partial Test".to_string(),
        format!("{}/partial.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 2, "Should only have 2 valid items");
    assert_eq!(items[0].title, "Valid Item");
    assert_eq!(items[1].title, "Another Valid Item");
}

#[tokio::test]
async fn test_source_trait_methods() {
    let source = RssSource::new(
        "Test Source".to_string(),
        "https://example.com/feed.xml".to_string(),
    );

    assert_eq!(source.name(), "Test Source");
    assert_eq!(source.url(), "https://example.com/feed.xml");
}

#[tokio::test]
async fn test_whitespace_normalization() {
    let mock_server = MockServer::start().await;
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <item>
      <title>  Title   with    extra     spaces  </title>
      <link>https://example.com/spaces</link>
      <description>
        Description
        with
        line
        breaks
        and     spaces
      </description>
    </item>
  </channel>
</rss>"#;

    Mock::given(method("GET"))
        .and(path("/whitespace.xml"))
        .respond_with(ResponseTemplate::new(200).set_body_string(rss))
        .mount(&mock_server)
        .await;

    let source = RssSource::new(
        "Whitespace Test".to_string(),
        format!("{}/whitespace.xml", mock_server.uri()),
    );

    let result = source.fetch().await;
    assert!(result.is_ok());

    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Title with extra spaces");
    assert_eq!(
        items[0].summary.as_ref().unwrap(),
        "Description with line breaks and spaces"
    );
}
