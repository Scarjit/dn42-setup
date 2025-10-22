#![allow(dead_code, unused_imports)]

mod config;
pub mod deploy;

pub use config::{BgpConfig, ChallengeConfig, InterfaceConfig, PeerConfig, WgConfig};
pub use deploy::{deploy_config, remove_config, WgKeypair};
