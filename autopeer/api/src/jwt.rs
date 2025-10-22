use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// JWT claims for authenticated ASNs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// ASN that was authenticated
    pub asn: u32,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
}

impl Claims {
    /// Create new claims for an ASN with default expiration (7 days)
    pub fn new(asn: u32) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::days(7);

        Claims {
            asn,
            iat: now.timestamp(),
            exp: expiration.timestamp(),
        }
    }
}

/// Generate a JWT token for an authenticated ASN
pub fn generate_token(asn: u32, secret: &str) -> Result<String, String> {
    let claims = Claims::new(asn);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Failed to generate token: {}", e))
}

/// Verify that a token is valid for a specific ASN
pub fn verify_token(token: &str, asn: u32, secret: &str) -> Result<(), String> {
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Failed to verify token: {}", e))?;

    if claims.asn != asn {
        return Err(format!(
            "Token ASN mismatch: expected {}, got {}",
            asn, claims.asn
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-for-jwt-testing";

    #[test]
    fn test_generate_and_verify_token() {
        let asn = 4242420257;
        let token = generate_token(asn, TEST_SECRET).unwrap();

        let result = verify_token(&token, asn, TEST_SECRET);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_token_asn_mismatch() {
        let asn = 4242420257;
        let wrong_asn = 4242421234;
        let token = generate_token(asn, TEST_SECRET).unwrap();

        let result = verify_token(&token, wrong_asn, TEST_SECRET);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token ASN mismatch"));
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let asn = 4242420257;
        let token = generate_token(asn, TEST_SECRET).unwrap();

        let result = verify_token(&token, asn, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_token() {
        let asn = 4242420257;
        let result = verify_token("invalid.token.here", asn, TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_new() {
        let asn = 4242420257;
        let claims = Claims::new(asn);

        assert_eq!(claims.asn, asn);
        // Default expiration is 7 days
        let expected_exp = claims.iat + (7 * 24 * 3600);
        assert_eq!(claims.exp, expected_exp);
    }
}
