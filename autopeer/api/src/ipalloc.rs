/// Derive IPv6 link-local addresses from ASNs
/// Format: fe80::{peer_asn}:{my_asn}:{0/1}/64
pub struct Ipv6LinkLocal {
    pub local: String,
    pub peer: String,
}

impl Ipv6LinkLocal {
    /// Generate IPv6 link-local addresses for a peering
    /// Format: fe80::{peer_short}:{my_short}:{0 for local, 1 for peer}
    pub fn from_asns(my_asn: u32, peer_asn: u32) -> Self {
        let my_short = my_asn % 10000; // Last 4 digits
        let peer_short = peer_asn % 10000; // Last 4 digits

        Ipv6LinkLocal {
            local: format!("fe80::{}:{}:0/64", peer_short, my_short),
            peer: format!("fe80::{}:{}:1", peer_short, my_short),
        }
    }

    /// Get the local address without CIDR
    pub fn local_addr(&self) -> String {
        self.local.split('/').next().unwrap().to_string()
    }
}

/// Generate WireGuard interface name from ASN
pub fn interface_name(asn: u32) -> String {
    format!("wg-as{}", asn)
}

/// Derive WireGuard port from ASN
/// Format: 3{last 4 digits of ASN}
pub fn wireguard_port(asn: u32) -> u16 {
    let short_asn = asn % 10000; // Last 4 digits
    30000 + (short_asn as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv6_from_asns() {
        // Your ASN: 4242420257, Peer ASN: 4242422225
        let ips = Ipv6LinkLocal::from_asns(4242420257, 4242422225);
        assert_eq!(ips.local, "fe80::2225:257:0/64");
        assert_eq!(ips.peer, "fe80::2225:257:1");
        assert_eq!(ips.local_addr(), "fe80::2225:257:0");
    }

    #[test]
    fn test_ipv6_from_asns_reverse() {
        // Your ASN: 4242420257, Peer ASN: 4242423088
        let ips = Ipv6LinkLocal::from_asns(4242420257, 4242423088);
        assert_eq!(ips.local, "fe80::3088:257:0/64");
        assert_eq!(ips.peer, "fe80::3088:257:1");
    }

    #[test]
    fn test_interface_name() {
        assert_eq!(interface_name(4242422225), "wg-as4242422225");
        assert_eq!(interface_name(4242423088), "wg-as4242423088");
    }

    #[test]
    fn test_wireguard_port() {
        assert_eq!(wireguard_port(4242422225), 32225);
        assert_eq!(wireguard_port(4242423088), 33088);
        assert_eq!(wireguard_port(4242421234), 31234);
    }
}
