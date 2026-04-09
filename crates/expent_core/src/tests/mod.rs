use super::{Core, CoreConfig};
use rstest::*;
use std::env;
use tracing_test::tracing_test;

// Fixture for a default, valid CoreConfig
#[fixture]
fn default_core_config() -> CoreConfig {
    CoreConfig {
        database_url: "sqlite::memory:".to_string(), // In-memory SQLite for testing
        s3_endpoint: "http://localhost:9000".to_string(),
        s3_access_key_id: "test_access_key".to_string(),
        s3_secret_access_key: "test_secret_key".to_string(),
        s3_bucket_name: "test_bucket".to_string(),
    }
}

// Ensure OCR_WORKER_URL is set for tests that need OcrService::new() to succeed
#[fixture]
fn setup_ocr_worker_url() {
    env::set_var("OCR_WORKER_URL", "http://localhost:8090");
}

#[rstest]
#[tokio::test]
#[tracing_test] // Enables tracing for the test
async fn test_core_init_happy_path(
    default_core_config: CoreConfig,
    setup_ocr_worker_url: (), // Use the fixture to set the env var
) {
    let core = Core::init(default_core_config).await;
    assert!(core.is_ok(), "Core::init should succeed on happy path");
    let core_instance = core.unwrap();

    // Assert specific inner values or characteristics
    assert!(!core_instance.db.is_closed(), "Database connection should be open");
    assert!(Arc::strong_count(&core_instance.auth) > 0, "Auth service should be initialized");
    // TODO: Add more specific assertions for upload_client and ocr_service if possible without
    // exposing internal details or requiring complex mocks
}

#[rstest]
#[tokio::test]
#[tracing_test]
async fn test_core_init_db_connection_failure() {
    let config = CoreConfig {
        database_url: "invalid_db_url".to_string(), // Invalid URL to trigger connection failure
        s3_endpoint: "http://localhost:9000".to_string(),
        s3_access_key_id: "test_access_key".to_string(),
        s3_secret_access_key: "test_secret_key".to_string(),
        s3_bucket_name: "test_bucket".to_string(),
    };

    let core = Core::init(config).await;
    assert!(core.is_err(), "Core::init should fail with invalid database URL");
    let error = core.unwrap_err();
    let error_message = error.to_string();
    assert!(
        error_message.contains("database URL"),
        "Error message should indicate database connection issue: {}",
        error_message
    );
}
