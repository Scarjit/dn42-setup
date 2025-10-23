/// Test helpers for API endpoint testing
///
/// These helpers allow testing API endpoints without actually running
/// system commands like wg-quick or birdc.

use crate::config::AppConfig;
use std::path::PathBuf;
use std::sync::Arc;

/// Create a test configuration with temporary directories
pub fn test_config_with_temp_dirs() -> (Arc<AppConfig>, tempfile::TempDir, tempfile::TempDir) {
    let pending_dir = tempfile::TempDir::new().unwrap();
    let verified_dir = tempfile::TempDir::new().unwrap();

    let config = Arc::new(AppConfig {
        registry: crate::config::RegistryConfig {
            url: "https://test.example".to_string(),
            path: PathBuf::from("./data/dn42-registry"),
            username: "test".to_string(),
            token: "test".to_string(),
        },
        jwt_secret: "test-secret-key-for-testing-at-least-32-chars-long".to_string(),
        my_asn: 4242420257,
        bind_address: "127.0.0.1:3000".to_string(),
        data_pending_dir: pending_dir.path().to_string_lossy().to_string(),
        data_verified_dir: verified_dir.path().to_string_lossy().to_string(),
        cookie_domains: vec!["localhost".to_string()],
    });

    (config, pending_dir, verified_dir)
}

/// Create a basic test configuration
pub fn test_config() -> Arc<AppConfig> {
    Arc::new(AppConfig {
        registry: crate::config::RegistryConfig {
            url: "https://test.example".to_string(),
            path: PathBuf::from("/tmp/test-registry"),
            username: "test".to_string(),
            token: "test".to_string(),
        },
        jwt_secret: "test-secret-key-for-testing-at-least-32-chars-long".to_string(),
        my_asn: 4242420257,
        bind_address: "127.0.0.1:3000".to_string(),
        data_pending_dir: "/tmp/test-pending".to_string(),
        data_verified_dir: "/tmp/test-verified".to_string(),
        cookie_domains: vec!["localhost".to_string()],
    })
}
