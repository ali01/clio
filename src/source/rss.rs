use crate::error::{ClioError, ErrorContext};
use crate::source::{Item, Source};
use async_trait::async_trait;
use atom_syndication::Feed as AtomFeed;
use chrono::{DateTime, Utc};
use html_escape::decode_html_entities;
use reqwest::Client;
use rss::Channel;
use std::time::Duration;
use uuid::Uuid;

/// RSS/Atom feed source implementation
#[derive(Debug, Clone)]
pub struct RssSource {
    name: String,
    url: String,
    client: Client,
}

#[async_trait]
impl Source for RssSource {
    async fn fetch(&self) -> Result<Vec<Item>, ClioError> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .clio_network_err(format!("Failed to pull feed from {}", self.url))?;

        if !response.status().is_success() {
            return Err(ClioError::Network(format!(
                "HTTP {} from {}",
                response.status(),
                self.url
            )));
        }

        let content = response
            .bytes()
            .await
            .clio_network_err("Failed to read response body")?;

        // Try parsing as RSS first
        if let Ok(items) = self.parse_rss(&content) {
            return Ok(items);
        }

        // Try parsing as Atom
        if let Ok(content_str) = std::str::from_utf8(&content) {
            if let Ok(items) = self.parse_atom(content_str) {
                return Ok(items);
            }
        }

        Err(ClioError::Parse(format!(
            "Failed to parse feed from {} as RSS or Atom",
            self.url
        )))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> &str {
        &self.url
    }
}

impl RssSource {
    /// Create a new RSS/Atom feed source
    pub fn new(name: String, url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Clio/0.1.0")
            .build()
            .unwrap_or_default();

        Self { name, url, client }
    }

    /// Parse RSS feed content
    fn parse_rss(&self, content: &[u8]) -> Result<Vec<Item>, ClioError> {
        let channel = Channel::read_from(content)?;
        let mut items = Vec::new();

        for rss_item in channel.items() {
            // Skip items without title or link
            let title = match rss_item.title() {
                Some(t) if !t.trim().is_empty() => decode_html_entities(t).to_string(),
                _ => continue,
            };

            let link = match rss_item.link() {
                Some(l) if !l.trim().is_empty() => l.to_string(),
                _ => continue,
            };

            let summary = rss_item
                .description()
                .map(|d| decode_html_entities(d).to_string())
                .map(|s| Self::normalize_whitespace(&s));

            let pub_date = rss_item.pub_date().and_then(|d| Self::parse_date(d).ok());

            items.push(Item {
                id: Uuid::new_v4().to_string(),
                source_name: self.name.clone(),
                title: Self::normalize_whitespace(&title),
                link,
                summary,
                pub_date,
            });
        }

        Ok(items)
    }

    /// Parse Atom feed content
    fn parse_atom(&self, content: &str) -> Result<Vec<Item>, ClioError> {
        let feed = content.parse::<AtomFeed>()?;
        let mut items = Vec::new();

        for entry in feed.entries() {
            // Skip entries without title
            let title = entry.title().value.trim();
            if title.is_empty() {
                continue;
            }

            // Get the link - prefer alternate links, fall back to first link
            let link = entry
                .links()
                .iter()
                .find(|l| l.rel() == "alternate")
                .or_else(|| entry.links().first())
                .map(|l| l.href().to_string());

            let link = match link {
                Some(l) if !l.trim().is_empty() => l,
                _ => continue,
            };

            let summary = entry
                .summary()
                .map(|s| decode_html_entities(&s.value).to_string())
                .or_else(|| {
                    entry
                        .content()
                        .and_then(|c| c.value())
                        .map(|v| decode_html_entities(v).to_string())
                })
                .map(|s| Self::normalize_whitespace(&s));

            let pub_date = entry
                .published()
                .or_else(|| Some(entry.updated()))
                .map(|d| DateTime::from_timestamp(d.timestamp(), 0).unwrap_or(Utc::now()));

            items.push(Item {
                id: Uuid::new_v4().to_string(),
                source_name: self.name.clone(),
                title: Self::normalize_whitespace(decode_html_entities(title).as_ref()),
                link,
                summary,
                pub_date,
            });
        }

        Ok(items)
    }

    /// Parse various date formats commonly used in feeds
    fn parse_date(date_str: &str) -> Result<DateTime<Utc>, ClioError> {
        // Try RFC 2822 format (common in RSS)
        if let Ok(dt) = DateTime::parse_from_rfc2822(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try RFC 3339 format (common in Atom)
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try some common variations
        let formats = [
            "%a, %d %b %Y %H:%M:%S %z", // RFC 2822 variant
            "%Y-%m-%dT%H:%M:%S%.fZ",    // ISO 8601 with fractional seconds
            "%Y-%m-%dT%H:%M:%SZ",       // ISO 8601 without fractional seconds
            "%Y-%m-%d %H:%M:%S",        // Common format without timezone
            "%d %b %Y %H:%M:%S %z",     // Another RSS variant
        ];

        for format in &formats {
            if let Ok(dt) = DateTime::parse_from_str(date_str, format) {
                return Ok(dt.with_timezone(&Utc));
            }
            // Try without timezone
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }

        Err(ClioError::Parse(format!(
            "Unable to parse date: {date_str}"
        )))
    }

    /// Normalize whitespace in text
    fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn create_test_source(url: &str) -> RssSource {
        RssSource::new("Test Source".to_string(), url.to_string())
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            RssSource::normalize_whitespace("  hello   world  "),
            "hello world"
        );
        assert_eq!(
            RssSource::normalize_whitespace("line\nbreak\ttab"),
            "line break tab"
        );
        assert_eq!(RssSource::normalize_whitespace("single"), "single");
    }

    #[test]
    fn test_parse_date_rfc2822() {
        let result = RssSource::parse_date("Wed, 01 Jan 2025 12:00:00 +0000");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_date_rfc3339() {
        let result = RssSource::parse_date("2025-01-01T12:00:00Z");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_date_iso8601_with_fraction() {
        let result = RssSource::parse_date("2025-01-01T12:00:00.123Z");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
    }

    #[test]
    fn test_parse_date_invalid() {
        let result = RssSource::parse_date("not a date");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClioError::Parse(_)));
    }

    #[tokio::test]
    async fn test_pull_rss_success() {
        let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <description>Test Description</description>
    <item>
      <title>Test Article</title>
      <link>https://example.com/article1</link>
      <description>Article description</description>
      <pubDate>Wed, 01 Jan 2025 12:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/feed.xml")
            .with_status(200)
            .with_body(rss_content)
            .create();
        let source = create_test_source(&format!("{}/feed.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Article");
        assert_eq!(items[0].link, "https://example.com/article1");
        assert_eq!(items[0].summary, Some("Article description".to_string()));
        assert!(items[0].pub_date.is_some());
    }

    #[tokio::test]
    async fn test_pull_atom_success() {
        let atom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Atom Feed</title>
  <link href="https://example.com"/>
  <updated>2025-01-01T12:00:00Z</updated>
  <entry>
    <title>Atom Article</title>
    <link href="https://example.com/atom-article"/>
    <summary>Atom article summary</summary>
    <published>2025-01-01T12:00:00Z</published>
  </entry>
</feed>"#;

        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/atom.xml")
            .with_status(200)
            .with_body(atom_content)
            .create();
        let source = create_test_source(&format!("{}/atom.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Atom Article");
        assert_eq!(items[0].link, "https://example.com/atom-article");
        assert_eq!(items[0].summary, Some("Atom article summary".to_string()));
        assert!(items[0].pub_date.is_some());
    }

    #[tokio::test]
    async fn test_pull_html_entities() {
        let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item>
      <title>Article &amp; Title &lt;with&gt; entities</title>
      <link>https://example.com/article</link>
      <description>Description with &quot;quotes&quot; and &apos;apostrophes&apos;</description>
    </item>
  </channel>
</rss>"#;

        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/entities.xml")
            .with_status(200)
            .with_body(rss_content)
            .create();
        let source = create_test_source(&format!("{}/entities.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Article & Title <with> entities");
        assert_eq!(
            items[0].summary,
            Some("Description with \"quotes\" and 'apostrophes'".to_string())
        );
    }

    #[tokio::test]
    async fn test_pull_skip_invalid_items() {
        let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <item>
      <title></title>
      <link>https://example.com/empty-title</link>
    </item>
    <item>
      <title>Valid Article</title>
      <link>https://example.com/valid</link>
    </item>
    <item>
      <title>No Link Article</title>
    </item>
  </channel>
</rss>"#;

        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/skip-invalid.xml")
            .with_status(200)
            .with_body(rss_content)
            .create();
        let source = create_test_source(&format!("{}/skip-invalid.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Valid Article");
    }

    #[tokio::test]
    async fn test_pull_http_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server.mock("GET", "/404.xml").with_status(404).create();
        let source = create_test_source(&format!("{}/404.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClioError::Network(_)));
    }

    #[tokio::test]
    async fn test_pull_malformed_xml() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/malformed.xml")
            .with_status(200)
            .with_body("not valid xml")
            .create();
        let source = create_test_source(&format!("{}/malformed.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClioError::Parse(_)));
    }

    #[tokio::test]
    async fn test_pull_empty_feed() {
        let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
    <link>https://example.com</link>
    <description>No items</description>
  </channel>
</rss>"#;

        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/empty.xml")
            .with_status(200)
            .with_body(rss_content)
            .create();

        let source = create_test_source(&format!("{}/empty.xml", server.url()));
        let result = source.fetch().await;

        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 0);
    }
}
