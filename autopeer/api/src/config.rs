use std::env;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub registry: RegistryConfig,
    pub jwt_secret: String,
    pub my_asn: u32,
    pub bind_address: String,
    pub data_pending_dir: String,
    pub data_verified_dir: String,
    pub cookie_domains: Vec<String>,
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
        let jwt_secret =
            env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set".to_string())?;

        let my_asn = env::var("MY_ASN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4242420257);

        let bind_address = env::var("BIND_ADDRESS")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string());

        let data_pending_dir = env::var("DATA_PENDING_DIR")
            .unwrap_or_else(|_| "./data/pending".to_string());

        let data_verified_dir = env::var("DATA_VERIFIED_DIR")
            .unwrap_or_else(|_| "./data/verified".to_string());

        let cookie_domains = env::var("COOKIE_DOMAINS")
            .unwrap_or_else(|_| "localhost".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(AppConfig {
            registry: RegistryConfig::from_env()?,
            jwt_secret,
            my_asn,
            bind_address,
            data_pending_dir,
            data_verified_dir,
            cookie_domains,
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
        assert!(!config.jwt_secret.is_empty());
    }
}
