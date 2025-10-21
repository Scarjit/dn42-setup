use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::env;
use std::path::{Path, PathBuf};

pub struct RegistryConfig {
    pub url: String,
    pub path: PathBuf,
    pub username: String,
    pub token: String,
}

impl RegistryConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let url = env::var("DN42_REGISTRY_URL")
            .unwrap_or_else(|_| "https://git.dn42.dev/dn42/registry".to_string());

        let path =
            env::var("DN42_REGISTRY_PATH").unwrap_or_else(|_| "./data/dn42-registry".to_string());

        let username =
            env::var("DN42_GIT_USERNAME").map_err(|_| "DN42_GIT_USERNAME not set".to_string())?;

        let token = env::var("DN42_GIT_TOKEN").map_err(|_| "DN42_GIT_TOKEN not set".to_string())?;

        Ok(RegistryConfig {
            url,
            path: PathBuf::from(path),
            username,
            token,
        })
    }
}

pub struct RegistrySync {
    config: RegistryConfig,
}

impl RegistrySync {
    pub fn new(config: RegistryConfig) -> Self {
        Self { config }
    }

    /// Clone or update the DN42 registry
    pub fn sync(&self) -> Result<(), String> {
        if self.config.path.exists() {
            self.pull()
        } else {
            self.clone()
        }
    }

    /// Clone the repository
    fn clone(&self) -> Result<(), String> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        let mut callbacks = RemoteCallbacks::new();
        let username = self.config.username.clone();
        let token = self.config.token.clone();

        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder
            .clone(&self.config.url, &self.config.path)
            .map_err(|e| format!("Failed to clone repository: {}", e))?;

        Ok(())
    }

    /// Pull latest changes
    fn pull(&self) -> Result<(), String> {
        let repo = Repository::open(&self.config.path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote: {}", e))?;

        let mut callbacks = RemoteCallbacks::new();
        let username = self.config.username.clone();
        let token = self.config.token.clone();

        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext(&username, &token)
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote
            .fetch(&["main"], Some(&mut fetch_options), None)
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        // Fast-forward merge
        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {}", e))?;

        let fetch_commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|e| format!("Failed to get commit: {}", e))?;

        let analysis = repo
            .merge_analysis(&[&fetch_commit])
            .map_err(|e| format!("Failed to analyze merge: {}", e))?;

        if analysis.0.is_up_to_date() {
            Ok(())
        } else if analysis.0.is_fast_forward() {
            let refname = "refs/heads/main";
            let mut reference = repo
                .find_reference(refname)
                .map_err(|e| format!("Failed to find reference: {}", e))?;

            reference
                .set_target(fetch_commit.id(), "Fast-forward")
                .map_err(|e| format!("Failed to set target: {}", e))?;

            repo.set_head(refname)
                .map_err(|e| format!("Failed to set HEAD: {}", e))?;

            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| format!("Failed to checkout: {}", e))?;

            Ok(())
        } else {
            Err("Cannot fast-forward, manual intervention required".to_string())
        }
    }

    /// Get the path to the registry
    pub fn registry_path(&self) -> &Path {
        &self.config.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_actual_sync() {
        // Load .env
        dotenvy::dotenv().ok();

        let config = RegistryConfig::from_env().expect("Failed to load config");
        let sync = RegistrySync::new(config);

        let result = sync.sync();
        assert!(result.is_ok(), "Sync failed: {:?}", result);

        // Verify registry path exists
        assert!(sync.registry_path().exists());
    }

    #[test]
    fn test_registry_config_from_env() {
        // Set test environment variables
        env::set_var("DN42_REGISTRY_URL", "https://test.example.com/registry.git");
        env::set_var("DN42_REGISTRY_PATH", "/tmp/test-registry");
        env::set_var("DN42_GIT_USERNAME", "testuser");
        env::set_var("DN42_GIT_TOKEN", "testtoken");

        let config = RegistryConfig::from_env().unwrap();
        assert_eq!(config.url, "https://test.example.com/registry.git");
        assert_eq!(config.path, PathBuf::from("/tmp/test-registry"));
        assert_eq!(config.username, "testuser");
        assert_eq!(config.token, "testtoken");

        // Clean up
        env::remove_var("DN42_REGISTRY_URL");
        env::remove_var("DN42_REGISTRY_PATH");
        env::remove_var("DN42_GIT_USERNAME");
        env::remove_var("DN42_GIT_TOKEN");
    }

    #[test]
    fn test_registry_config_defaults() {
        // Remove all env vars to test defaults
        env::remove_var("DN42_REGISTRY_URL");
        env::remove_var("DN42_REGISTRY_PATH");
        env::remove_var("DN42_GIT_USERNAME");
        env::remove_var("DN42_GIT_TOKEN");

        let result = RegistryConfig::from_env();
        assert!(result.is_err()); // Should fail without username/token
    }
}
