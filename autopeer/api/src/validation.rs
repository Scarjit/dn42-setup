use axum::http::StatusCode;
use once_cell::sync::Lazy;
use regex::Regex;

// Compile regexes once at startup
static IPV4_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d{1,3}\.){3}\d{1,3}:\d{1,5}$").unwrap()
});

static IPV6_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\[([0-9a-fA-F:]+)\]:\d{1,5}$").unwrap()
});

/// Validate ASN is within DN42 range
pub fn validate_asn(asn: u32) -> Result<(), (StatusCode, String)> {
    if asn < 4200000000 || asn > 4294967294 {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("ASN {} is out of valid DN42 range (4200000000-4294967294)", asn),
        ));
    }
    Ok(())
}

/// Validate endpoint format (IP:port)
pub fn validate_endpoint(endpoint: &str) -> Result<(), (StatusCode, String)> {
    if !IPV4_PATTERN.is_match(endpoint) && !IPV6_PATTERN.is_match(endpoint) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid endpoint format. Expected IP:port or [IPv6]:port".to_string(),
        ));
    }

    // Check port is not 0
    let port_str = endpoint.rsplit(':').next().unwrap();
    if port_str == "0" || port_str == "00000" {
        return Err((StatusCode::BAD_REQUEST, "Port cannot be 0".to_string()));
    }

    Ok(())
}

/// Validate WireGuard public key format (base64, 44 chars)
pub fn validate_wg_pubkey(key: &str) -> Result<(), (StatusCode, String)> {
    if key.len() != 44 {
        return Err((
            StatusCode::BAD_REQUEST,
            "WireGuard public key must be 44 characters".to_string(),
        ));
    }

    // Check if it's valid base64
    if !key.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=') {
        return Err((
            StatusCode::BAD_REQUEST,
            "WireGuard public key must be valid base64".to_string(),
        ));
    }

    Ok(())
}

/// Validate PGP public key format (basic check - actual parsing happens later)
pub fn validate_pgp_key(key: &str) -> Result<(), (StatusCode, String)> {
    if key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "PGP key cannot be empty".to_string()));
    }
    // Real validation happens in verify_key_fingerprint
    Ok(())
}

/// Validate signed challenge format (basic check - actual parsing happens later)
pub fn validate_signed_challenge(signed: &str) -> Result<(), (StatusCode, String)> {
    if signed.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Signed message cannot be empty".to_string()));
    }
    // Real validation happens in gpg::verify_signature
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_asn_valid() {
        assert!(validate_asn(4242420257).is_ok());
        assert!(validate_asn(4200000000).is_ok());
        assert!(validate_asn(4294967294).is_ok());
    }

    #[test]
    fn test_validate_asn_invalid() {
        assert!(validate_asn(100).is_err());
        assert!(validate_asn(4199999999).is_err());
        assert!(validate_asn(4294967295).is_err());
    }

    #[test]
    fn test_validate_endpoint_ipv4() {
        assert!(validate_endpoint("192.168.1.1:51820").is_ok());
        assert!(validate_endpoint("1.2.3.4:12345").is_ok());
    }

    #[test]
    fn test_validate_endpoint_ipv6() {
        assert!(validate_endpoint("[2001:db8::1]:51820").is_ok());
        assert!(validate_endpoint("[fe80::1]:12345").is_ok());
    }

    #[test]
    fn test_validate_endpoint_invalid() {
        assert!(validate_endpoint("not-an-ip:1234").is_err());
        assert!(validate_endpoint("192.168.1.1").is_err());
        assert!(validate_endpoint("192.168.1.1:0").is_err());
    }

    #[test]
    fn test_validate_wg_pubkey_valid() {
        let key = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQR";
        assert_eq!(key.len(), 44);
        assert!(validate_wg_pubkey(key).is_ok());
    }

    #[test]
    fn test_validate_wg_pubkey_invalid() {
        assert!(validate_wg_pubkey("tooshort").is_err());
        assert!(validate_wg_pubkey("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ").is_err());
    }
}
