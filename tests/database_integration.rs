use std::env;
use std::sync::Arc;

use clio::config::SupabaseConfig;
use clio::database::{Database, SupabaseClient};
use clio::error::ClioError;
use serial_test::serial;

/// Mock client that simulates Supabase responses for integration testing
#[derive(Debug)]
struct IntegrationMockClient {
    url: String,
    queries: std::sync::Mutex<Vec<String>>,
    table_exists_count: std::sync::Mutex<usize>,
    error_on_query: Option<String>,
}

impl IntegrationMockClient {
    fn new(url: String) -> Self {
        Self {
            url,
            queries: std::sync::Mutex::new(Vec::new()),
            table_exists_count: std::sync::Mutex::new(0),
            error_on_query: None,
        }
    }

    fn with_error(url: String, error_msg: String) -> Self {
        Self {
            url,
            queries: std::sync::Mutex::new(Vec::new()),
            table_exists_count: std::sync::Mutex::new(0),
            error_on_query: Some(error_msg),
        }
    }

    #[allow(dead_code)]
    fn get_queries(&self) -> Vec<String> {
        self.queries.lock().unwrap().clone()
    }
}

impl SupabaseClient for IntegrationMockClient {
    fn execute(&self, query: &str) -> Result<(), ClioError> {
        if let Some(ref error_msg) = self.error_on_query {
            if query.contains("CREATE TABLE") || query.contains("SELECT 1") {
                return Err(ClioError::Database(error_msg.clone()));
            }
        }

        self.queries.lock().unwrap().push(query.to_string());
        Ok(())
    }

    fn table_exists(&self, _table_name: &str) -> Result<bool, ClioError> {
        if let Some(ref error_msg) = self.error_on_query {
            if error_msg.contains("table_check") {
                return Err(ClioError::Database(error_msg.clone()));
            }
        }

        let mut count = self.table_exists_count.lock().unwrap();
        *count += 1;
        // First call returns false (table doesn't exist)
        Ok(*count > 1)
    }

    fn url(&self) -> &str {
        &self.url
    }
}

#[test]
fn test_database_integration_with_mock_client() {
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client.clone());

    // Test schema initialization
    let result = db.init_schema();
    assert!(result.is_ok(), "Schema initialization should succeed");

    // Verify correct queries were executed
    let queries = mock_client.get_queries();
    assert!(!queries.is_empty(), "Should have executed queries");
    assert!(
        queries[0].contains("CREATE TABLE IF NOT EXISTS items"),
        "Should create items table"
    );
}

#[test]
fn test_database_integration_connection_verification() {
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client.clone());

    // Test connection verification
    let result = db.verify_connection();
    assert!(result.is_ok(), "Connection verification should succeed");

    // Verify SELECT 1 was executed
    let queries = mock_client.get_queries();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0], "SELECT 1");
}

#[test]
fn test_database_integration_error_handling() {
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::with_error(
        "https://test.supabase.co".to_string(),
        "Connection lost".to_string(),
    ));
    let db = Database::with_client(config, mock_client);

    // Test that errors are properly propagated
    let result = db.verify_connection();
    assert!(result.is_err(), "Should fail with connection error");

    let err = result.unwrap_err();
    assert!(matches!(err, ClioError::Database(_)));
    assert!(err.to_string().contains("Connection lost"));
}

#[test]
fn test_database_integration_schema_already_exists() {
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));

    // Call table_exists once to make it return true on next call
    let _ = mock_client.table_exists("items");

    let db = Database::with_client(config, mock_client.clone());

    // Initialize schema when table already exists
    let result = db.init_schema();
    assert!(result.is_ok(), "Should succeed even if table exists");

    // No CREATE TABLE query should be executed
    let queries = mock_client.get_queries();
    assert_eq!(queries.len(), 0, "Should not create table if it exists");
}

#[test]
#[serial]
fn test_database_integration_environment_variables() {
    // This test verifies that environment variable validation works correctly
    // We use a controlled environment to avoid interfering with actual env vars

    // Save current env vars
    let old_url = env::var("SUPABASE_URL").ok();
    let old_key = env::var("SUPABASE_SECRET_KEY").ok();

    // Test missing URL
    unsafe {
        env::remove_var("SUPABASE_URL");
        env::remove_var("SUPABASE_SECRET_KEY");
    }
    let result = Database::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("SUPABASE_URL"));

    // Test missing key
    unsafe {
        env::set_var("SUPABASE_URL", "https://test.supabase.co");
        env::remove_var("SUPABASE_SECRET_KEY");
    }
    let result = Database::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("SUPABASE_SECRET_KEY"));

    // Test invalid URL scheme
    unsafe {
        env::set_var("SUPABASE_URL", "http://test.supabase.co");
        env::set_var("SUPABASE_SECRET_KEY", "sb_secret_test");
    }
    let result = Database::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HTTPS"));

    // Restore original env vars
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
}

#[test]
fn test_database_integration_table_structure() {
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client.clone());

    // Initialize schema
    let _ = db.init_schema();

    // Verify table structure
    let queries = mock_client.get_queries();
    let create_table_query = &queries[0];

    // Check all required columns are present
    assert!(create_table_query.contains("id UUID PRIMARY KEY"));
    assert!(create_table_query.contains("source_name TEXT NOT NULL"));
    assert!(create_table_query.contains("title TEXT NOT NULL"));
    assert!(create_table_query.contains("link TEXT NOT NULL UNIQUE"));
    assert!(create_table_query.contains("summary TEXT"));
    assert!(create_table_query.contains("pub_date TIMESTAMPTZ"));
    assert!(create_table_query.contains("is_read BOOLEAN DEFAULT FALSE"));
    assert!(create_table_query.contains("created_at TIMESTAMPTZ DEFAULT NOW()"));
    assert!(create_table_query.contains("updated_at TIMESTAMPTZ DEFAULT NOW()"));

    // Check indexes are created
    assert!(queries.len() >= 4, "Should create table and 3 indexes");
    assert!(queries[1].contains("idx_items_pub_date"));
    assert!(queries[2].contains("idx_items_created_at"));
    assert!(queries[3].contains("idx_items_is_read"));
}

#[test]
fn test_database_integration_retry_logic() {
    // Test that transient failures can be handled
    // This would be more relevant with actual retry logic implementation
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client);

    // For now, just verify connection works
    let result = db.verify_connection();
    assert!(result.is_ok());
}

#[test]
fn test_database_integration_concurrent_access() {
    // Test that the database can handle concurrent operations
    use std::thread;

    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_test123".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client.clone());

    // Spawn multiple threads to access the database
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let db_clone = db.clone();
            thread::spawn(move || db_clone.verify_connection())
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        let result = handle.join().expect("Thread should not panic");
        assert!(result.is_ok(), "Each thread should succeed");
    }

    // Verify all connections were tested
    let queries = mock_client.get_queries();
    assert_eq!(queries.len(), 5, "Should have 5 SELECT 1 queries");
}

#[test]
fn test_database_integration_secret_key_protection() {
    // Ensure secret keys are never exposed in logs or debug output
    let config = SupabaseConfig {
        url: "https://test.supabase.co".to_string(),
        secret_key: "sb_secret_very_secret_key_12345".to_string(),
    };

    let mock_client = Arc::new(IntegrationMockClient::new(
        "https://test.supabase.co".to_string(),
    ));
    let db = Database::with_client(config, mock_client);

    // Check debug output doesn't contain the secret
    let debug_output = format!("{:?}", db);
    assert!(
        !debug_output.contains("sb_secret_very_secret_key_12345"),
        "Secret key should not appear in debug output"
    );
    assert!(
        !debug_output.contains("very_secret_key_12345"),
        "Secret key substring should not appear in debug output"
    );
}
