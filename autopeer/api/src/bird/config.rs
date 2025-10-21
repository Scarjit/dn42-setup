use crate::ipalloc::Ipv6LinkLocal;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

/// BIRD BGP peer configuration
pub struct BirdPeerConfig {
    pub my_asn: u32,
    pub peer_asn: u32,
    pub peer_name: String,
    pub interface_name: String,
    pub ips: Ipv6LinkLocal,
}

impl BirdPeerConfig {
    /// Create a new BIRD peer configuration
    pub fn new(my_asn: u32, peer_asn: u32, peer_name: String, interface_name: String) -> Self {
        let ips = Ipv6LinkLocal::from_asns(my_asn, peer_asn);

        BirdPeerConfig {
            my_asn,
            peer_asn,
            peer_name,
            interface_name,
            ips,
        }
    }

    /// Generate BIRD configuration as string using Tera template
    pub fn to_config(&self) -> Result<String, String> {
        // Load template
        let mut tera = Tera::default();
        let template = include_str!("peer.conf.tera");
        tera.add_raw_template("peer.conf", template)
            .map_err(|e| format!("Failed to parse template: {}", e))?;

        // Create context
        let mut context = Context::new();
        context.insert("my_asn", &self.my_asn);
        context.insert("peer_asn", &self.peer_asn);
        context.insert("peer_name", &self.peer_name);
        context.insert("interface_name", &self.interface_name);
        context.insert("local_ip", &self.ips.local_addr());
        context.insert("peer_ip", &self.ips.peer);

        // Render template
        tera.render("peer.conf", &context)
            .map_err(|e| format!("Failed to render template: {}", e))
    }

    /// Write configuration to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let config = self.to_config()?;
        fs::write(path, config).map_err(|e| format!("Failed to write BIRD config: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bird_config_generation() {
        let config = BirdPeerConfig::new(
            4242420257,
            4242421816,
            "potat0.cc FRA".to_string(),
            "wg-as4242421816".to_string(),
        );

        let bird_conf = config.to_config().unwrap();

        // Check key elements
        assert!(bird_conf.contains("protocol bgp autopeer_as4242421816 from dnpeers"));
        assert!(bird_conf.contains("local fe80::1816:257:0 as 4242420257"));
        assert!(bird_conf.contains("neighbor fe80::1816:257:1 as 4242421816"));
        assert!(bird_conf.contains("interface \"wg-as4242421816\""));
        assert!(bird_conf.contains("extended next hop yes"));
        assert!(bird_conf.contains("AutoPeer - potat0.cc FRA - AS4242421816"));
    }

    #[test]
    fn test_protocol_name_format() {
        let config = BirdPeerConfig::new(
            4242420257,
            4242422225,
            "Test Peer".to_string(),
            "wg-as4242422225".to_string(),
        );

        let bird_conf = config.to_config().unwrap();
        assert!(bird_conf.contains("protocol bgp autopeer_as4242422225"));
    }

    #[test]
    fn test_bird_config_ipv6_format() {
        // Verify the new IP format: fe80::{peer}:{my}:{0/1}
        let config = BirdPeerConfig::new(
            4242420257,
            4242422225,
            "Test".to_string(),
            "wg-as4242422225".to_string(),
        );

        let bird_conf = config.to_config().unwrap();
        // Local should be fe80::2225:257:0
        // Peer should be fe80::2225:257:1
        assert!(bird_conf.contains("local fe80::2225:257:0 as 4242420257"));
        assert!(bird_conf.contains("neighbor fe80::2225:257:1 as 4242422225"));
    }
}
