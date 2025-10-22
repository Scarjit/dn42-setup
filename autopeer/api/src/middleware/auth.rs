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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::generate_token;
    use axum::http::{Request, StatusCode};
    use tower_cookies::CookieManagerLayer;
    use tower::ServiceExt;
    use axum::{body::Body, Router, routing::get};

    /// Helper to create a test AppConfig
    fn test_config() -> Arc<AppConfig> {
        Arc::new(AppConfig {
            registry: crate::config::RegistryConfig {
                url: "https://test.example".to_string(),
                path: std::path::PathBuf::from("/tmp/test"),
                username: "test".to_string(),
                token: "test".to_string(),
            },
            jwt_secret: "test-secret-key-for-testing".to_string(),
            my_asn: 4242420257,
            bind_address: "127.0.0.1:3000".to_string(),
            data_pending_dir: "/tmp/pending".to_string(),
            data_verified_dir: "/tmp/verified".to_string(),
            cookie_domains: vec!["localhost".to_string()],
        })
    }

    /// Test handler that requires JWT authentication
    async fn test_handler(auth: JwtAuth) -> String {
        format!("Authenticated as ASN {}", auth.asn)
    }

    #[tokio::test]
    async fn test_jwt_auth_success() {
        let config = test_config();
        let test_asn = 4242421234;

        // Generate a valid JWT token
        let token = generate_token(test_asn, &config.jwt_secret).unwrap();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with the token cookie
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add cookie header
        let cookie_header = format!("autopeer_token={}", token);
        request.headers_mut().insert(
            axum::http::header::COOKIE,
            cookie_header.parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should succeed
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_jwt_auth_missing_cookie() {
        let config = test_config();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request WITHOUT a cookie
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should fail with 401
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_invalid_token() {
        let config = test_config();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with an invalid token
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        request.headers_mut().insert(
            axum::http::header::COOKIE,
            "autopeer_token=invalid.token.here".parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should fail with 401
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_wrong_secret() {
        let config = test_config();
        let test_asn = 4242421234;

        // Generate a token with a DIFFERENT secret
        let token = generate_token(test_asn, "wrong-secret").unwrap();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with the token signed with wrong secret
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let cookie_header = format!("autopeer_token={}", token);
        request.headers_mut().insert(
            axum::http::header::COOKIE,
            cookie_header.parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should fail with 401
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_empty_cookie_value() {
        let config = test_config();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with an empty cookie value
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        request.headers_mut().insert(
            axum::http::header::COOKIE,
            "autopeer_token=".parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should fail with 401
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_malformed_jwt() {
        let config = test_config();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with a malformed JWT (not enough parts)
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        request.headers_mut().insert(
            axum::http::header::COOKIE,
            "autopeer_token=not.a.valid.jwt.too.many.parts".parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should fail with 401
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_extracts_correct_asn() {
        let config = test_config();
        let test_asn = 4242429999;

        // Generate a valid JWT token
        let token = generate_token(test_asn, &config.jwt_secret).unwrap();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with the token cookie
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let cookie_header = format!("autopeer_token={}", token);
        request.headers_mut().insert(
            axum::http::header::COOKIE,
            cookie_header.parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should succeed
        assert_eq!(response.status(), StatusCode::OK);

        // Check that the response contains the correct ASN
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, format!("Authenticated as ASN {}", test_asn));
    }

    #[tokio::test]
    async fn test_jwt_auth_multiple_cookies() {
        let config = test_config();
        let test_asn = 4242421234;

        // Generate a valid JWT token
        let token = generate_token(test_asn, &config.jwt_secret).unwrap();

        // Create a test app with the handler
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(CookieManagerLayer::new())
            .with_state(config);

        // Create a request with multiple cookies including autopeer_token
        let mut request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let cookie_header = format!("other_cookie=value; autopeer_token={}; another=cookie", token);
        request.headers_mut().insert(
            axum::http::header::COOKIE,
            cookie_header.parse().unwrap(),
        );

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Should succeed and extract the correct cookie
        assert_eq!(response.status(), StatusCode::OK);
    }
}
