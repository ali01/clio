use crate::config::SupabaseConfig;
use crate::error::{ClioError, ErrorContext};
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

/// Database client wrapper for Supabase PostgreSQL connection
#[derive(Debug, Clone)]
pub struct Database {
    #[expect(dead_code)]
    config: Arc<SupabaseConfig>,
    client: Arc<dyn SupabaseClient>,
}

/// Trait for Supabase client operations (allows mocking in tests)
pub trait SupabaseClient: Send + Sync + std::fmt::Debug {
    /// Execute a query against the database
    fn execute(&self, query: &str) -> Result<(), ClioError>;

    /// Check if a table exists
    fn table_exists(&self, table_name: &str) -> Result<bool, ClioError>;

    /// Get the connection URL (for display/debugging, not the actual secret)
    fn url(&self) -> &str;
}

impl Database {
    /// Create a new database connection using environment variables
    pub fn new() -> Result<Self, ClioError> {
        let config = SupabaseConfig::from_env()?;
        let client = create_client(&config)?;

        Ok(Self {
            config: Arc::new(config),
            client,
        })
    }

    /// Create a database connection with a custom client (for testing)
    #[doc(hidden)]
    pub fn with_client(config: SupabaseConfig, client: Arc<dyn SupabaseClient>) -> Self {
        Self {
            config: Arc::new(config),
            client,
        }
    }

    /// Initialize the database schema if it doesn't exist
    pub fn init_schema(&self) -> Result<(), ClioError> {
        // Check if the items table exists
        if !self.client.table_exists("items")? {
            self.create_schema()?;
        }
        Ok(())
    }

    /// Create the database schema
    fn create_schema(&self) -> Result<(), ClioError> {
        // Create the items table with all required columns
        let create_table_query = r#"
            CREATE TABLE IF NOT EXISTS items (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                source_name TEXT NOT NULL,
                title TEXT NOT NULL,
                link TEXT NOT NULL UNIQUE,
                summary TEXT,
                pub_date TIMESTAMPTZ,
                is_read BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#;

        self.client.execute(create_table_query)
            .clio_database_err("Failed to create items table")?;

        // Create indexes for efficient querying
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_items_pub_date ON items(pub_date DESC)",
            "CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at DESC)",
            "CREATE INDEX IF NOT EXISTS idx_items_is_read ON items(is_read)",
        ];

        for index_query in &indexes {
            self.client.execute(index_query)
                .clio_database_err(format!("Failed to create index: {index_query}"))?;
        }

        Ok(())
    }

    /// Verify the database connection is working
    pub fn verify_connection(&self) -> Result<(), ClioError> {
        // Try a simple query to verify the connection works
        self.client.execute("SELECT 1")
            .clio_database_err("Failed to verify database connection")?;
        Ok(())
    }
}

/// Real Supabase client implementation using HTTP REST API
#[derive(Debug, Clone)]
struct RealSupabaseClient {
    client: Client,
    base_url: String,
    secret_key: String,
}

impl RealSupabaseClient {
    fn new(config: &SupabaseConfig) -> Result<Self, ClioError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ClioError::Database(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.url().to_string(),
            secret_key: config.secret_key().to_string(),
        })
    }

    async fn execute_query(&self, query: &str) -> Result<(), ClioError> {
        // For DDL operations, we use the Supabase SQL endpoint
        let url = format!("{}/rest/v1/rpc/query", self.base_url);

        let response = self.client
            .post(&url)
            .header("apikey", &self.secret_key)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": query
            }))
            .send()
            .await
            .map_err(|e| ClioError::Database(format!("Failed to execute query: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ClioError::Database(format!(
                "Query execution failed with status {}: {}",
                status, error_text
            )));
        }

        Ok(())
    }

    async fn check_table_exists(&self, table_name: &str) -> Result<bool, ClioError> {
        // Query the information_schema to check if table exists
        let url = format!("{}/rest/v1/rpc/table_exists", self.base_url);

        let response = self.client
            .post(&url)
            .header("apikey", &self.secret_key)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "table_name": table_name
            }))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status() == StatusCode::OK => {
                // Table checking function exists and returned result
                let exists = resp.json::<bool>().await.unwrap_or(false);
                Ok(exists)
            }
            Ok(resp) if resp.status() == StatusCode::NOT_FOUND => {
                // The RPC function doesn't exist, try direct table query
                self.check_table_via_select(table_name).await
            }
            Ok(resp) => {
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(ClioError::Database(format!("Failed to check table existence: {}", error_text)))
            }
            Err(e) => Err(ClioError::Database(format!("Failed to check table existence: {}", e))),
        }
    }

    async fn check_table_via_select(&self, table_name: &str) -> Result<bool, ClioError> {
        // Try to query the table directly
        let url = format!("{}/rest/v1/{}", self.base_url, table_name);

        let response = self.client
            .head(&url)
            .header("apikey", &self.secret_key)
            .header("Authorization", format!("Bearer {}", self.secret_key))
            .send()
            .await
            .map_err(|e| ClioError::Database(format!("Failed to check table: {}", e)))?;

        // If we get OK or NO_CONTENT, table exists
        // If we get NOT_FOUND, table doesn't exist
        match response.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND | StatusCode::NOT_ACCEPTABLE => Ok(false),
            status => {
                Err(ClioError::Database(format!(
                    "Unexpected status when checking table: {}",
                    status
                )))
            }
        }
    }
}

impl SupabaseClient for RealSupabaseClient {
    fn execute(&self, query: &str) -> Result<(), ClioError> {
        // Block on the async operation
        // In a real implementation, we'd make everything async, but for now this works
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ClioError::Database(format!("Failed to create runtime: {}", e)))?;
        runtime.block_on(self.execute_query(query))
    }

    fn table_exists(&self, table_name: &str) -> Result<bool, ClioError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ClioError::Database(format!("Failed to create runtime: {}", e)))?;
        runtime.block_on(self.check_table_exists(table_name))
    }

    fn url(&self) -> &str {
        &self.base_url
    }
}

/// Create a real Supabase client
fn create_client(config: &SupabaseConfig) -> Result<Arc<dyn SupabaseClient>, ClioError> {
    // For testing, we can check if we should return a mock
    if cfg!(test) && config.url().contains("test.supabase.co") {
        // In test mode with test URL, return error to force use of mock
        return Err(ClioError::Database("Use mock client in tests".to_string()));
    }

    let client = RealSupabaseClient::new(config)?;
    Ok(Arc::new(client))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;
    #[cfg(test)]
    use serial_test::serial;

    /// Mock Supabase client for testing
    #[derive(Debug)]
    struct MockSupabaseClient {
        url: String,
        queries_executed: Mutex<Vec<String>>,
        table_exists_responses: Mutex<Vec<bool>>,
        should_fail: bool,
        failure_message: String,
    }

    impl MockSupabaseClient {
        fn new(url: String) -> Self {
            Self {
                url,
                queries_executed: Mutex::new(Vec::new()),
                table_exists_responses: Mutex::new(vec![false]), // Default: table doesn't exist
                should_fail: false,
                failure_message: String::new(),
            }
        }

        fn with_failure(url: String, message: String) -> Self {
            Self {
                url,
                queries_executed: Mutex::new(Vec::new()),
                table_exists_responses: Mutex::new(Vec::new()),
                should_fail: true,
                failure_message: message,
            }
        }

        fn with_existing_table(url: String) -> Self {
            Self {
                url,
                queries_executed: Mutex::new(Vec::new()),
                table_exists_responses: Mutex::new(vec![true]), // Table exists
                should_fail: false,
                failure_message: String::new(),
            }
        }

        #[allow(dead_code)]
        fn get_executed_queries(&self) -> Vec<String> {
            self.queries_executed.lock().unwrap().clone()
        }
    }

    impl SupabaseClient for MockSupabaseClient {
        fn execute(&self, query: &str) -> Result<(), ClioError> {
            if self.should_fail {
                return Err(ClioError::Database(self.failure_message.clone()));
            }

            self.queries_executed.lock().unwrap().push(query.to_string());
            Ok(())
        }

        fn table_exists(&self, _table_name: &str) -> Result<bool, ClioError> {
            if self.should_fail {
                return Err(ClioError::Database(self.failure_message.clone()));
            }

            let mut responses = self.table_exists_responses.lock().unwrap();
            if responses.is_empty() {
                Ok(false)
            } else {
                Ok(responses.remove(0))
            }
        }

        fn url(&self) -> &str {
            &self.url
        }
    }

    // Helper to set environment variables safely for tests
    fn with_env_vars<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Save current values
        let old_url = env::var("SUPABASE_URL").ok();
        let old_key = env::var("SUPABASE_SECRET_KEY").ok();

        // Run the test
        let result = f();

        // Restore old values
        unsafe {
            match old_url {
                Some(val) => env::set_var("SUPABASE_URL", val),
                None => env::remove_var("SUPABASE_URL"),
            }
            match old_key {
                Some(val) => env::set_var("SUPABASE_SECRET_KEY", val),
                None => env::remove_var("SUPABASE_SECRET_KEY"),
            }
        }

        result
    }

    #[test]
    #[serial]
    fn test_database_new_missing_url() {
        with_env_vars(|| {
            unsafe {
                env::remove_var("SUPABASE_URL");
                env::remove_var("SUPABASE_SECRET_KEY");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("SUPABASE_URL"));
        })
    }

    #[test]
    #[serial]
    fn test_database_new_missing_secret_key() {
        with_env_vars(|| {
            unsafe {
                env::set_var("SUPABASE_URL", "https://test.supabase.co");
                env::remove_var("SUPABASE_SECRET_KEY");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("SUPABASE_SECRET_KEY"));
        })
    }

    #[test]
    #[serial]
    fn test_database_new_invalid_url() {
        with_env_vars(|| {
            unsafe {
                env::set_var("SUPABASE_URL", "not-a-url");
                env::set_var("SUPABASE_SECRET_KEY", "sb_secret_test123");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("Invalid SUPABASE_URL"));
        })
    }

    #[test]
    #[serial]
    fn test_database_new_http_url() {
        with_env_vars(|| {
            unsafe {
                env::set_var("SUPABASE_URL", "http://test.supabase.co");
                env::set_var("SUPABASE_SECRET_KEY", "sb_secret_test123");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("HTTPS"));
        })
    }

    #[test]
    #[serial]
    fn test_database_new_invalid_secret_key_prefix() {
        with_env_vars(|| {
            unsafe {
                env::set_var("SUPABASE_URL", "https://test.supabase.co");
                env::set_var("SUPABASE_SECRET_KEY", "invalid_key");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("sb_secret_"));
        })
    }

    #[test]
    #[serial]
    fn test_database_new_empty_secret_key() {
        with_env_vars(|| {
            unsafe {
                env::set_var("SUPABASE_URL", "https://test.supabase.co");
                env::set_var("SUPABASE_SECRET_KEY", "");
            }

            let result = Database::new();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ClioError::Config(_)));
            assert!(err.to_string().contains("cannot be empty"));
        })
    }

    #[test]
    fn test_init_schema_creates_table_when_not_exists() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::new("https://test.supabase.co".to_string()));
        let db = Database::with_client(config, mock_client.clone());

        let result = db.init_schema();
        assert!(result.is_ok());

        // Verify that CREATE TABLE and indexes were executed
        let queries = mock_client.get_executed_queries();
        assert!(queries.len() >= 4); // CREATE TABLE + 3 indexes
        assert!(queries[0].contains("CREATE TABLE IF NOT EXISTS items"));
        assert!(queries[1].contains("CREATE INDEX IF NOT EXISTS idx_items_pub_date"));
        assert!(queries[2].contains("CREATE INDEX IF NOT EXISTS idx_items_created_at"));
        assert!(queries[3].contains("CREATE INDEX IF NOT EXISTS idx_items_is_read"));
    }

    #[test]
    fn test_init_schema_skips_when_table_exists() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::with_existing_table(
            "https://test.supabase.co".to_string()
        ));
        let db = Database::with_client(config, mock_client.clone());

        let result = db.init_schema();
        assert!(result.is_ok());

        // Verify that no queries were executed (table already exists)
        let queries = mock_client.get_executed_queries();
        assert_eq!(queries.len(), 0);
    }

    #[test]
    fn test_create_schema_creates_correct_table_structure() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::new("https://test.supabase.co".to_string()));
        let db = Database::with_client(config, mock_client.clone());

        let result = db.create_schema();
        assert!(result.is_ok());

        let queries = mock_client.get_executed_queries();

        // Verify table structure
        let create_table = &queries[0];
        assert!(create_table.contains("id UUID PRIMARY KEY"));
        assert!(create_table.contains("source_name TEXT NOT NULL"));
        assert!(create_table.contains("title TEXT NOT NULL"));
        assert!(create_table.contains("link TEXT NOT NULL UNIQUE"));
        assert!(create_table.contains("summary TEXT"));
        assert!(create_table.contains("pub_date TIMESTAMPTZ"));
        assert!(create_table.contains("is_read BOOLEAN DEFAULT FALSE"));
        assert!(create_table.contains("created_at TIMESTAMPTZ DEFAULT NOW()"));
        assert!(create_table.contains("updated_at TIMESTAMPTZ DEFAULT NOW()"));
    }

    #[test]
    fn test_create_schema_handles_execution_error() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::with_failure(
            "https://test.supabase.co".to_string(),
            "Connection refused".to_string(),
        ));
        let db = Database::with_client(config, mock_client);

        let result = db.create_schema();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ClioError::Database(_)));
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_verify_connection_success() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::new("https://test.supabase.co".to_string()));
        let db = Database::with_client(config, mock_client.clone());

        let result = db.verify_connection();
        assert!(result.is_ok());

        // Verify SELECT 1 was executed
        let queries = mock_client.get_executed_queries();
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0], "SELECT 1");
    }

    #[test]
    fn test_verify_connection_failure() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::with_failure(
            "https://test.supabase.co".to_string(),
            "Network timeout".to_string(),
        ));
        let db = Database::with_client(config, mock_client);

        let result = db.verify_connection();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ClioError::Database(_)));
        assert!(err.to_string().contains("Network timeout"));
    }

    #[test]
    fn test_table_exists_check() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::new("https://test.supabase.co".to_string()));
        let _db = Database::with_client(config, mock_client.clone());

        // First call returns false (default)
        let exists = mock_client.table_exists("items").unwrap();
        assert!(!exists);

        // Mock client with existing table
        let mock_client2 = Arc::new(MockSupabaseClient::with_existing_table(
            "https://test.supabase.co".to_string()
        ));
        let exists2 = mock_client2.table_exists("items").unwrap();
        assert!(exists2);
    }

    #[test]
    fn test_init_schema_handles_network_errors() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::with_failure(
            "https://test.supabase.co".to_string(),
            "Connection timeout".to_string(),
        ));
        let db = Database::with_client(config, mock_client);

        let result = db.init_schema();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ClioError::Database(_)));
        assert!(err.to_string().contains("Connection timeout"));
    }

    #[test]
    fn test_secret_key_not_exposed_in_debug() {
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_supersecret123456".to_string(),
        };

        let mock_client = Arc::new(MockSupabaseClient::new("https://test.supabase.co".to_string()));
        let db = Database::with_client(config, mock_client);

        // Debug format should not contain the secret key
        let debug_str = format!("{:?}", db);
        assert!(!debug_str.contains("sb_secret_supersecret123456"));
        assert!(!debug_str.contains("supersecret123456"));
    }

    #[test]
    fn test_create_client_returns_error_for_test_url() {
        // In test mode, create_client returns an error for test URLs to force mock usage
        let config = SupabaseConfig {
            url: "https://test.supabase.co".to_string(),
            secret_key: "sb_secret_test123".to_string(),
        };

        let result = create_client(&config);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, ClioError::Database(_)));
        assert!(err.to_string().contains("Use mock client in tests"));
    }

    #[test]
    fn test_create_real_client_with_non_test_url() {
        // With a non-test URL, it should create a real client
        let config = SupabaseConfig {
            url: "https://myproject.supabase.co".to_string(),
            secret_key: "sb_secret_real123".to_string(),
        };

        let result = create_client(&config);
        assert!(result.is_ok());

        let client = result.unwrap();
        assert_eq!(client.url(), "https://myproject.supabase.co");
    }
}
