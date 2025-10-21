use crate::config::RegistryConfig;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::Path;
use std::sync::Mutex;

static REGISTRY_LOCK: Mutex<()> = Mutex::new(());

pub struct RegistrySync {
    config: RegistryConfig,
}

impl RegistrySync {
    pub fn new(config: RegistryConfig) -> Self {
        Self { config }
    }

    /// Clone or update the DN42 registry (thread-safe)
    pub fn sync(&self) -> Result<(), String> {
        // Acquire lock to ensure only one thread syncs at a time
        let _lock = REGISTRY_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        if self.config.path.exists() {
            // Try to pull, if it fails remove and re-clone
            match self.pull() {
                Ok(()) => Ok(()),
                Err(_e) => {
                    // Remove corrupted repository
                    std::fs::remove_dir_all(&self.config.path)
                        .map_err(|e| format!("Failed to remove corrupted repo: {}", e))?;
                    // Re-clone
                    self.clone()
                }
            }
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
    use std::path::PathBuf;

    #[test]
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
    fn test_registry_sync_with_custom_config() {
        // Create config directly without using env vars
        // Uses same path as test_actual_sync, but thread-safe thanks to mutex
        dotenvy::dotenv().ok();

        // Load from env for credentials, but create config directly
        let config = RegistryConfig::new(
            "https://git.dn42.dev/dn42/registry".to_string(),
            PathBuf::from("./data/dn42-registry"),
            std::env::var("DN42_GIT_USERNAME").unwrap(),
            std::env::var("DN42_GIT_TOKEN").unwrap(),
        );

        let sync = RegistrySync::new(config);
        let result = sync.sync();
        assert!(result.is_ok(), "Sync failed: {:?}", result);
        assert!(sync.registry_path().exists());
    }
}
