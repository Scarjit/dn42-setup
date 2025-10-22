mod config;
pub mod deploy;

pub use config::BirdPeerConfig;
pub use deploy::{deploy_config, remove_config};
