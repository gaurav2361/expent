use env_logger;
use expent_core::{Core, CoreConfig};
use rstest::*;
use std::env;
use std::sync::Arc;
use tracing_test::traced_test; // Corrected import

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
    unsafe {
        env::set_var("OCR_WORKER_URL", "http://localhost:8090");
    }
}

#[rstest]
#[tokio::test]
#[traced_test] // Corrected macro
#[allow(unused_variables)] // Allow unused for fixture that sets env var
async fn test_core_init_happy_path(
    default_core_config: CoreConfig,
    setup_ocr_worker_url: (), // Use the fixture to set the env var
) {
    let core = Core::init(default_core_config).await;
    assert!(
        core.is_ok(),
        "Core::init should succeed on happy path, but got {:?}",
        core.err()
    );
    let core_instance = core.unwrap();

    // Assert specific inner values or characteristics
    // sea_orm::DatabaseConnection does not have an `is_closed` method that is public.
    // We rely on the fact that `Database::connect` returning `Ok` means a connection was established.
    assert!(
        Arc::strong_count(&core_instance.auth) > 0,
        "Auth service should be initialized"
    );
    // As mentioned previously, direct assertions for internal states of UploadClient and OcrService
    // without exposing their internals or using mocks is challenging.
    // The successful creation of the Core struct implies these services were at least
    // instantiated without immediate errors.
}

#[rstest]
#[tokio::test]
#[traced_test] // Corrected macro
async fn test_core_init_db_connection_failure() {
    let _ = env_logger::builder().is_test(true).try_init(); // Initialize logger for this test

    let config = CoreConfig {
        database_url: "invalid_db_url".to_string(), // Invalid URL to trigger connection failure
        s3_endpoint: "http://localhost:9000".to_string(),
        s3_access_key_id: "test_access_key".to_string(),
        s3_secret_access_key: "test_secret_key".to_string(),
        s3_bucket_name: "test_bucket".to_string(),
    };

    // Ensure OCR_WORKER_URL is set even for this failing test, as OcrService::new() is called regardless
    unsafe {
        env::set_var("OCR_WORKER_URL", "http://localhost:8090");
    }

    let core = Core::init(config).await;
    assert!(
        core.is_err(),
        "Core::init should fail with invalid database URL"
    );

    // Use match to avoid Debug requirement on Core
    let error_message = match core {
        Ok(_) => panic!("Core::init unexpectedly succeeded"),
        Err(e) => e.to_string(),
    };

    assert!(
        error_message.contains("Connection Error") || error_message.contains("cannot be parsed"),
        "Error message should indicate database connection issue: {}",
        error_message
    );
}
