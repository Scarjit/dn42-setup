use crate::config::AppConfig;
use crate::jwt::verify_token;
use axum::http::StatusCode;

/// Validated JWT token with ASN
#[derive(Clone, Debug)]
pub struct JwtAuth {
    pub asn: u32,
}

impl JwtAuth {
    /// Verify a JWT token for a given ASN
    pub fn verify(token: &str, asn: u32, config: &AppConfig) -> Result<Self, (StatusCode, String)> {
        verify_token(token, asn, &config.jwt_secret)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Token verification failed: {}", e)))?;

        Ok(JwtAuth { asn })
    }
}
