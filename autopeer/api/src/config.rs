use std::env;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub registry: RegistryConfig,
}

/// DN42 Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub url: String,
    pub path: PathBuf,
    pub username: String,
    pub token: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        Ok(AppConfig {
            registry: RegistryConfig::from_env()?,
        })
    }
}

impl RegistryConfig {
    /// Load registry configuration from environment variables
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

    /// Create a new registry configuration (for testing)
    pub fn new(url: String, path: PathBuf, username: String, token: String) -> Self {
        RegistryConfig {
            url,
            path,
            username,
            token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_config_from_env() {
        dotenvy::dotenv().ok();

        let config = RegistryConfig::from_env().unwrap();
        assert_eq!(config.url, "https://git.dn42.dev/dn42/registry");
        assert!(!config.username.is_empty());
        assert!(!config.token.is_empty());
    }

    #[test]
    fn test_app_config_from_env() {
        dotenvy::dotenv().ok();

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.registry.url, "https://git.dn42.dev/dn42/registry");
    }
}
