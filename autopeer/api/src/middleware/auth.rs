use crate::config::AppConfig;
use crate::jwt::decode_token;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use std::sync::Arc;
use tower_cookies::Cookies;

/// Validated JWT token with ASN extracted from cookie
#[derive(Clone, Debug)]
pub struct JwtAuth {
    pub asn: u32,
}

impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
    Arc<AppConfig>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let config = Arc::<AppConfig>::from_ref(state);
        // Extract cookies
        let cookies = Cookies::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read cookies".to_string()))?;

        // Get token from cookie
        let token = cookies
            .get("autopeer_token")
            .ok_or((StatusCode::UNAUTHORIZED, "No authentication cookie".to_string()))?
            .value()
            .to_string();

        // Decode JWT to get ASN
        let asn = decode_token(&token, &config.jwt_secret)
            .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Token decode failed: {}", e)))?;

        Ok(JwtAuth { asn })
    }
}
