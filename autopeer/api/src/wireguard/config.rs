#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

/// WireGuard interface configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterfaceConfig {
    pub address: Vec<String>,
    pub private_key: String,
    pub listen_port: u16,
    pub table: Option<String>,
}

/// WireGuard peer configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerConfig {
    pub public_key: String,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive: Option<u16>,
}

/// Custom Challenge section for autopeer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChallengeConfig {
    pub code: String,
    pub asn: u32,
}

/// Custom BGP section for autopeer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BgpConfig {
    pub mpbgp: bool,
    pub extended_next_hop: bool,
    pub local: String,
    pub neighbor: String,
}

/// Complete WireGuard configuration file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WgConfig {
    pub interface: InterfaceConfig,
    pub peer: Option<PeerConfig>,
    pub challenge: Option<ChallengeConfig>,
    pub bgp: Option<BgpConfig>,
}

impl WgConfig {
    /// Parse a WireGuard config file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Self::from_string(&content)
    }

    /// Parse a WireGuard config from string
    pub fn from_string(content: &str) -> Result<Self, String> {
        let sections = parse_ini_sections(content)?;

        // Parse Interface section (required)
        let interface = parse_interface(&sections)?;

        // Parse Peer section (optional)
        let peer = parse_peer(&sections).ok();

        // Parse Challenge section (optional)
        let challenge = parse_challenge(&sections).ok();

        // Parse BGP section (optional)
        let bgp = parse_bgp(&sections).ok();

        Ok(WgConfig {
            interface,
            peer,
            challenge,
            bgp,
        })
    }

    /// Write config to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = self.as_string()?;
        fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Convert config to string representation using Tera template
    fn as_string(&self) -> Result<String, String> {
        // Load template
        let mut tera = Tera::default();
        let template = include_str!("wg.conf.tera");
        tera.add_raw_template("wg.conf", template)
            .map_err(|e| format!("Failed to parse template: {}", e))?;

        // Create context
        let mut context = Context::new();
        context.insert("interface_address", &self.interface.address);
        context.insert("interface_private_key", &self.interface.private_key);
        context.insert("interface_listen_port", &self.interface.listen_port);
        context.insert("interface_table", &self.interface.table);
        context.insert("peer", &self.peer);
        context.insert("challenge", &self.challenge);
        context.insert("bgp", &self.bgp);

        // Render template
        tera.render("wg.conf", &context)
            .map_err(|e| format!("Failed to render template: {}", e))
    }
}

impl fmt::Display for WgConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_string() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "Error rendering config: {}", e),
        }
    }
}

/// Parse INI-style sections from config content
fn parse_ini_sections(
    content: &str,
) -> Result<HashMap<String, HashMap<String, Vec<String>>>, String> {
    let mut sections: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].to_string());
            sections.insert(current_section.clone().unwrap(), HashMap::new());
            continue;
        }

        // Key-value pair
        if let Some(section_name) = &current_section {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                sections
                    .get_mut(section_name)
                    .unwrap()
                    .entry(key)
                    .or_default()
                    .push(value);
            }
        }
    }

    Ok(sections)
}

fn parse_interface(
    sections: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Result<InterfaceConfig, String> {
    let section = sections
        .get("Interface")
        .ok_or("Missing [Interface] section")?;

    let address = section
        .get("Address")
        .ok_or("Missing Address in [Interface]")?
        .clone();

    let private_key = section
        .get("PrivateKey")
        .and_then(|v| v.first())
        .ok_or("Missing PrivateKey in [Interface]")?
        .clone();

    let listen_port = section
        .get("ListenPort")
        .and_then(|v| v.first())
        .ok_or("Missing ListenPort in [Interface]")?
        .parse::<u16>()
        .map_err(|e| format!("Invalid ListenPort: {}", e))?;

    let table = section.get("Table").and_then(|v| v.first()).cloned();

    Ok(InterfaceConfig {
        address,
        private_key,
        listen_port,
        table,
    })
}

fn parse_peer(
    sections: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Result<PeerConfig, String> {
    let section = sections.get("Peer").ok_or("Missing [Peer] section")?;

    let public_key = section
        .get("PublicKey")
        .and_then(|v| v.first())
        .ok_or("Missing PublicKey in [Peer]")?
        .clone();

    let endpoint = section.get("Endpoint").and_then(|v| v.first()).cloned();

    let allowed_ips = section
        .get("AllowedIPs")
        .ok_or("Missing AllowedIPs in [Peer]")?
        .clone();

    let persistent_keepalive = section
        .get("PersistentKeepalive")
        .and_then(|v| v.first())
        .and_then(|s| s.parse::<u16>().ok());

    Ok(PeerConfig {
        public_key,
        endpoint,
        allowed_ips,
        persistent_keepalive,
    })
}

fn parse_challenge(
    sections: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Result<ChallengeConfig, String> {
    let section = sections
        .get("Challenge")
        .ok_or("Missing [Challenge] section")?;

    let code = section
        .get("Code")
        .and_then(|v| v.first())
        .ok_or("Missing Code in [Challenge]")?
        .clone();

    let asn = section
        .get("ASN")
        .and_then(|v| v.first())
        .ok_or("Missing ASN in [Challenge]")?
        .parse::<u32>()
        .map_err(|e| format!("Invalid ASN: {}", e))?;

    Ok(ChallengeConfig { code, asn })
}

fn parse_bgp(
    sections: &HashMap<String, HashMap<String, Vec<String>>>,
) -> Result<BgpConfig, String> {
    let section = sections.get("BGP").ok_or("Missing [BGP] section")?;

    let mpbgp = section
        .get("MPBGP")
        .and_then(|v| v.first())
        .map(|s| s.to_lowercase() == "on" || s.to_lowercase() == "true")
        .unwrap_or(false);

    let extended_next_hop = section
        .get("ExtendedNextHop")
        .and_then(|v| v.first())
        .map(|s| s.to_lowercase() == "true" || s.to_lowercase() == "on")
        .unwrap_or(false);

    let local = section
        .get("Local")
        .and_then(|v| v.first())
        .ok_or("Missing Local in [BGP]")?
        .clone();

    let neighbor = section
        .get("Neighbor")
        .and_then(|v| v.first())
        .ok_or("Missing Neighbor in [BGP]")?
        .clone();

    Ok(BgpConfig {
        mpbgp,
        extended_next_hop,
        local,
        neighbor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let config_str = r#"
[Interface]
Address = fe80::2225:1/64
PrivateKey = MA3Oj1xzJzoGfIkMJagCXOHmGIkLkK49XUFfqS1Xjmo=
ListenPort = 51827
Table = off

[Peer]
PublicKey = uS1AYe7zTGAP48XeNn0vppNjg7q0hawyh8Y0bvvAWhk=
Endpoint = dn42-de.maraun.de:20257
AllowedIPs = 172.20.0.0/14
AllowedIPs = fd00::/8
PersistentKeepalive = 25
"#;

        let config = WgConfig::from_string(config_str).unwrap();
        assert_eq!(config.interface.listen_port, 51827);
        assert_eq!(config.interface.address, vec!["fe80::2225:1/64"]);
        assert!(config.peer.is_some());
        assert_eq!(
            config.peer.as_ref().unwrap().endpoint,
            Some("dn42-de.maraun.de:20257".to_string())
        );
    }

    #[test]
    fn test_parse_with_custom_sections() {
        let config_str = r#"
[Interface]
Address = fe80::1/64
PrivateKey = test123
ListenPort = 31234

[Challenge]
Code = AUTOPEER-4242421234-abc123
ASN = 4242421234

[BGP]
MPBGP = on
ExtendedNextHop = true
Local = fe80::1
Neighbor = fe80::2
"#;

        let config = WgConfig::from_string(config_str).unwrap();
        assert!(config.challenge.is_some());
        assert_eq!(config.challenge.as_ref().unwrap().asn, 4242421234);
        assert!(config.bgp.is_some());
        assert!(config.bgp.as_ref().unwrap().mpbgp);
    }

    #[test]
    fn test_config_roundtrip() {
        let original = WgConfig {
            interface: InterfaceConfig {
                address: vec!["fe80::1/64".to_string()],
                private_key: "testkey123".to_string(),
                listen_port: 31234,
                table: Some("off".to_string()),
            },
            peer: None,
            challenge: Some(ChallengeConfig {
                code: "AUTOPEER-TEST".to_string(),
                asn: 4242421234,
            }),
            bgp: None,
        };

        let serialized = original.as_string().unwrap();
        let parsed = WgConfig::from_string(&serialized).unwrap();

        assert_eq!(original, parsed);
    }
}
