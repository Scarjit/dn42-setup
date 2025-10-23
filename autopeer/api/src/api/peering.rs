use crate::bird;
use crate::challenge::{gpg::verify_signature, Challenge};
use crate::config::AppConfig;
use crate::ipalloc::{interface_name, wireguard_port, Ipv6LinkLocal};
use crate::jwt::generate_token;
use crate::middleware::JwtAuth;
use crate::registry::{get_pgp_fingerprint_for_asn, verify_key_fingerprint};
use crate::validation;
use crate::wireguard::{self, BgpConfig, InterfaceConfig, PeerConfig, WgConfig, WgKeypair};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info, warn};

/// Request to initialize a new peering
#[derive(Debug, Deserialize, Serialize)]
pub struct InitRequest {
    /// The peer's ASN
    pub asn: u32,
}

/// Response from peering initialization
#[derive(Debug, Deserialize, Serialize)]
pub struct InitResponse {
    /// The challenge code to sign
    pub challenge: String,
    /// The GPG key fingerprint from DN42 registry
    pub pgp_fingerprint: String,
}

/// POST /peering/init - Initialize a new peering
pub async fn init_peering(
    State(config): State<Arc<AppConfig>>,
    Json(req): Json<InitRequest>,
) -> Result<Json<InitResponse>, (StatusCode, String)> {
    info!("Peering init request for ASN {}", req.asn);

    // Validate ASN
    validation::validate_asn(req.asn)?;

    // Generate challenge
    let challenge = Challenge::generate(req.asn);

    // Fetch PGP fingerprint from registry (required)
    let registry_path = &config.registry.path;
    let pgp_fingerprint = get_pgp_fingerprint_for_asn(registry_path, req.asn)
        .map_err(|e| {
            error!("Failed to get PGP fingerprint for ASN {}: {}", req.asn, e);
            (StatusCode::BAD_REQUEST, format!("No GPG key found in DN42 registry for ASN {}: {}", req.asn, e))
        })?;

    // Store only the challenge (no keypair yet)
    let iface_name = interface_name(req.asn);
    let challenge_path = format!("{}/{}.conf", config.data_pending_dir, iface_name);

    // Ensure pending directory exists
    std::fs::create_dir_all(&config.data_pending_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create pending dir: {}", e)))?;

    // Save challenge to file
    std::fs::write(&challenge_path, &challenge.code)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save challenge: {}", e)))?;

    Ok(Json(InitResponse {
        challenge: challenge.code,
        pgp_fingerprint,
    }))
}

/// Request to verify a peering
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyRequest {
    /// The peer's ASN
    pub asn: u32,
    /// The signed challenge (cleartext signed message)
    pub signed_challenge: String,
    /// The peer's PGP public key
    pub public_key: String,
}

/// Deployment information (safe to show to user)
#[derive(Debug, Deserialize, Serialize)]
pub struct DeploymentInfo {
    /// Our WireGuard interface address
    pub interface_address: String,
    /// Our WireGuard listen port
    pub listen_port: u16,
    /// Our WireGuard public key (safe to expose)
    pub our_public_key: String,
    /// Our WireGuard endpoint
    pub our_endpoint: String,
    /// BGP configuration
    pub bgp_neighbor: String,
    pub bgp_local_as: u32,
    pub bgp_remote_as: u32,
    /// Whether the peering is currently active on the router
    pub is_active: bool,
}

/// Response from peering verification
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyResponse {
    /// JWT token for authenticated operations
    pub token: String,
}

/// POST /peering/verify - Verify a signed challenge and issue JWT
pub async fn verify_peering(
    State(config): State<Arc<AppConfig>>,
    cookies: Cookies,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, String)> {
    info!("Peering verify request for ASN {}", req.asn);

    // Validate inputs
    validation::validate_asn(req.asn)?;
    validation::validate_pgp_key(&req.public_key)?;
    validation::validate_signed_challenge(&req.signed_challenge)?;

    // Load pending challenge
    let iface_name = interface_name(req.asn);
    let challenge_path = format!("{}/{}.conf", config.data_pending_dir, iface_name);

    let stored_challenge = std::fs::read_to_string(&challenge_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Challenge not found: {}", e)))?;

    // Verify GPG signature
    let signature_valid = verify_signature(&stored_challenge, &req.signed_challenge, &req.public_key)
        .map_err(|e| {
            warn!("Signature verification failed for ASN {}: {}", req.asn, e);
            (StatusCode::UNAUTHORIZED, format!("Signature verification failed: {}", e))
        })?;

    if !signature_valid {
        warn!("Invalid signature for ASN {}", req.asn);
        return Err((StatusCode::UNAUTHORIZED, "Invalid signature".to_string()));
    }

    // Verify public key matches DN42 registry
    let registry_path = &config.registry.path;
    let expected_fingerprint = get_pgp_fingerprint_for_asn(registry_path, req.asn)
        .map_err(|e| {
            error!("Failed to get registry fingerprint for ASN {}: {}", req.asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get registry fingerprint: {}", e))
        })?;

    verify_key_fingerprint(&req.public_key, &expected_fingerprint)
        .map_err(|e| {
            warn!("Key verification failed for ASN {}: {}", req.asn, e);
            (StatusCode::UNAUTHORIZED, format!("Key verification failed: {}", e))
        })?;

    info!("Successfully verified ASN {}, issuing JWT token", req.asn);

    // Generate JWT token
    let token = generate_token(req.asn, &config.jwt_secret)
        .map_err(|e| {
            error!("Failed to generate token for ASN {}: {}", req.asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate token: {}", e))
        })?;

    // Remove pending challenge file
    let _ = std::fs::remove_file(&challenge_path);

    // Set HTTP-only secure cookies for each domain
    for domain in &config.cookie_domains {
        let mut cookie = Cookie::new("autopeer_token", token.clone());
        cookie.set_domain(domain.clone());
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_secure(true);
        cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
        cookie.set_max_age(tower_cookies::cookie::time::Duration::days(7));
        cookies.add(cookie);
    }

    Ok(Json(VerifyResponse {
        token,
    }))
}

/// Request to deploy a peering
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployRequest {
    /// The peer's WireGuard public key
    pub wg_public_key: String,
    /// The peer's public endpoint (IP:port)
    pub endpoint: String,
}

/// Response from peering deployment
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployResponse {
    /// Deployment information about what we configured
    pub deployment: DeploymentInfo,
}

/// POST /peering/deploy - Deploy a verified peering configuration
pub async fn deploy_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
    Json(req): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering deploy request for ASN {}", asn);

    // Validate WireGuard inputs
    validation::validate_wg_pubkey(&req.wg_public_key)?;
    validation::validate_endpoint(&req.endpoint)?;

    // Generate WireGuard keypair for this peer
    info!("Generating WireGuard keypair for ASN {}", asn);
    let keypair = WgKeypair::generate()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate keypair: {}", e)))?;

    // Allocate IPs
    let ips = Ipv6LinkLocal::from_asns(config.my_asn, asn);

    // Create complete WireGuard config
    let iface_name = interface_name(asn);
    let wg_config = WgConfig {
        interface: InterfaceConfig {
            address: vec![ips.peer.clone()],
            private_key: keypair.private_key.clone(),
            listen_port: wireguard_port(asn),
            table: Some("off".to_string()),
        },
        peer: Some(PeerConfig {
            public_key: req.wg_public_key.clone(),
            endpoint: Some(req.endpoint.clone()),
            allowed_ips: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            persistent_keepalive: Some(25),
        }),
        challenge: None,
        bgp: Some(BgpConfig {
            mpbgp: true,
            extended_next_hop: true,
            local: ips.local_addr(),
            neighbor: ips.peer.clone(),
        }),
    };

    // Save to verified directory
    let verified_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);
    std::fs::create_dir_all(&config.data_verified_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create verified dir: {}", e)))?;

    wg_config
        .to_file(&verified_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save verified config: {}", e)))?;

    // Generate WireGuard config string for deployment
    let wg_config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate WireGuard config: {}", e)))?;

    // Deploy WireGuard configuration
    info!("Deploying WireGuard config for ASN {} ({})", asn, iface_name);
    wireguard::deploy::deploy_config(&wg_config_str, &iface_name)
        .map_err(|e| {
            error!("Failed to deploy WireGuard for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deploy WireGuard: {}", e))
        })?;

    // Generate and deploy BIRD configuration
    info!("Deploying BIRD config for ASN {}", asn);
    let bird_peer_config = bird::BirdPeerConfig::new(
        config.my_asn,
        asn,
        format!("AS{}", asn),
        iface_name.clone(),
    );

    let bird_config_str = bird_peer_config
        .to_config()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate BIRD config: {}", e)))?;

    bird::deploy::deploy_config(&bird_config_str, asn)
        .map_err(|e| {
            error!("Failed to deploy BIRD config for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deploy BIRD config: {}", e))
        })?;

    info!("Successfully deployed peering for ASN {}", asn);

    // Build deployment info to return
    // Check if interface is active
    let is_active = wireguard::deploy::is_interface_active(&iface_name);

    let deployment = DeploymentInfo {
        interface_address: ips.local_addr(),
        listen_port: wireguard_port(asn),
        our_public_key: keypair.public_key.clone(),
        our_endpoint: format!("{}:{}", config.public_endpoint, wireguard_port(asn)),
        bgp_neighbor: ips.peer.clone(),
        bgp_local_as: config.my_asn,
        bgp_remote_as: asn,
        is_active,
    };

    Ok(Json(DeployResponse {
        deployment,
    }))
}

/// Response from config retrieval
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigResponse {
    /// The WireGuard configuration
    pub wireguard_config: String,
}

/// GET /peering/config - Retrieve verified peering configuration (ASN from JWT)
pub async fn get_config(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
) -> Result<Json<ConfigResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Config retrieval request for ASN {}", asn);

    // Load verified config
    let iface_name = interface_name(asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    let wg_config = WgConfig::from_file(&config_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Config not found: {}", e)))?;

    // Generate config string
    let config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate config: {}", e)))?;

    Ok(Json(ConfigResponse {
        wireguard_config: config_str,
    }))
}

/// GET /peering/status - Get deployment status (safe info only, no private keys)
/// Returns 404 if not deployed yet (user is logged in but hasn't provided WG details)
pub async fn get_status(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
) -> Result<Json<DeploymentInfo>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Status request for ASN {}", asn);

    // Load verified config (returns 404 if not deployed yet)
    let iface_name = interface_name(asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    let wg_config = WgConfig::from_file(&config_path)
        .map_err(|_| (StatusCode::NOT_FOUND, "Deployment not found. Please provide WireGuard details to deploy.".to_string()))?;

    // Extract safe info from config
    let ips = Ipv6LinkLocal::from_asns(config.my_asn, asn);

    // Derive public key from private key
    let public_key = WgKeypair::derive_public_key(&wg_config.interface.private_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to derive public key: {}", e)))?;

    // Check if interface is active
    let is_active = wireguard::deploy::is_interface_active(&iface_name);

    let deployment = DeploymentInfo {
        interface_address: ips.local_addr(),
        listen_port: wireguard_port(asn),
        our_public_key: public_key,
        our_endpoint: format!("{}:{}", config.public_endpoint, wireguard_port(asn)),
        bgp_neighbor: ips.peer.clone(),
        bgp_local_as: config.my_asn,
        bgp_remote_as: asn,
        is_active,
    };

    Ok(Json(deployment))
}

/// Request to update a peering configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateRequest {
    /// New endpoint (optional)
    pub endpoint: Option<String>,
}

/// Response from peering update
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateResponse {
    /// Status message
    pub status: String,
}

/// PATCH /peering/update - Update and re-deploy peering configuration
pub async fn update_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
    Json(req): Json<UpdateRequest>,
) -> Result<Json<UpdateResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering update request for ASN {}", asn);

    // Validate endpoint if provided
    if let Some(ref endpoint) = req.endpoint {
        validation::validate_endpoint(endpoint)?;
    }

    // Load verified config
    let iface_name = interface_name(asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    let mut wg_config = WgConfig::from_file(&config_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Config not found: {}", e)))?;

    // Update endpoint if provided
    if let Some(endpoint) = req.endpoint {
        if let Some(ref mut peer) = wg_config.peer {
            peer.endpoint = Some(endpoint);
        } else {
            return Err((StatusCode::BAD_REQUEST, "No peer configuration to update".to_string()));
        }
    }

    // Save updated config
    wg_config
        .to_file(&config_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save config: {}", e)))?;

    // Generate config string
    let wg_config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate config: {}", e)))?;

    // Re-deploy WireGuard
    info!("Re-deploying WireGuard config for ASN {} ({})", asn, iface_name);

    // First remove old config
    if let Err(e) = wireguard::deploy::remove_config(&iface_name) {
        warn!("Failed to remove old WireGuard config for ASN {}: {}", asn, e);
    }

    // Deploy new config
    wireguard::deploy::deploy_config(&wg_config_str, &iface_name)
        .map_err(|e| {
            error!("Failed to re-deploy WireGuard for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to re-deploy WireGuard: {}", e))
        })?;

    info!("Successfully updated peering for ASN {}", asn);

    Ok(Json(UpdateResponse {
        status: "updated".to_string(),
    }))
}

/// DELETE /peering - Delete peering configuration
pub async fn delete_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
) -> Result<Json<UpdateResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering delete request for ASN {}", asn);

    let iface_name = interface_name(asn);

    // Remove WireGuard config
    wireguard::deploy::remove_config(&iface_name)
        .map_err(|e| {
            error!("Failed to remove WireGuard for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove WireGuard: {}", e))
        })?;

    // Remove BIRD config
    bird::deploy::remove_config(asn)
        .map_err(|e| {
            error!("Failed to remove BIRD config for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove BIRD config: {}", e))
        })?;

    // Remove verified config file
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);
    std::fs::remove_file(&config_path)
        .map_err(|e| {
            error!("Failed to remove config file for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove config file: {}", e))
        })?;

    info!("Successfully deleted peering for ASN {}", asn);

    Ok(Json(UpdateResponse {
        status: "deleted".to_string(),
    }))
}

/// POST /peering/activate - Activate peering (copy config from verified dir to /etc/wireguard and deploy)
pub async fn activate_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
) -> Result<Json<UpdateResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering activate request for ASN {}", asn);

    let iface_name = interface_name(asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    // Check if config exists in verified directory
    if !std::path::Path::new(&config_path).exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("No peering configuration found for ASN {}. Please deploy first.", asn),
        ));
    }

    // Load config
    let wg_config = WgConfig::from_file(&config_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load config: {}", e)))?;

    // Generate WireGuard config string for deployment
    let wg_config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate WireGuard config: {}", e)))?;

    // Deploy WireGuard
    info!("Activating WireGuard config for ASN {} ({})", asn, iface_name);
    wireguard::deploy::deploy_config(&wg_config_str, &iface_name)
        .map_err(|e| {
            error!("Failed to activate WireGuard for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to activate WireGuard: {}", e))
        })?;

    // Deploy BIRD config if BGP is configured
    if let Some(ref bgp_config) = wg_config.bgp {
        info!("Activating BIRD config for ASN {}", asn);

        // Generate BIRD configuration
        let bird_peer_config = bird::BirdPeerConfig::new(
            config.my_asn,
            asn,
            format!("AS{}", asn),
            iface_name.clone(),
        );

        let bird_config_str = bird_peer_config
            .to_config()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate BIRD config: {}", e)))?;

        bird::deploy::deploy_config(&bird_config_str, asn)
            .map_err(|e| {
                error!("Failed to activate BIRD config for ASN {}: {}", asn, e);
                // Try to rollback WireGuard
                let _ = wireguard::deploy::remove_config(&iface_name);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to activate BIRD config: {}", e))
            })?;
    }

    info!("Successfully activated peering for ASN {}", asn);

    Ok(Json(UpdateResponse {
        status: "activated".to_string(),
    }))
}

/// POST /peering/deactivate - Deactivate peering (remove from /etc/wireguard but keep config in verified dir)
pub async fn deactivate_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
) -> Result<Json<UpdateResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering deactivate request for ASN {}", asn);

    let iface_name = interface_name(asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    // Check if config exists in verified directory
    if !std::path::Path::new(&config_path).exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("No peering configuration found for ASN {}", asn),
        ));
    }

    // Remove WireGuard config (this also brings down the interface)
    info!("Deactivating WireGuard config for ASN {} ({})", asn, iface_name);
    wireguard::deploy::remove_config(&iface_name)
        .map_err(|e| {
            error!("Failed to deactivate WireGuard for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deactivate WireGuard: {}", e))
        })?;

    // Remove BIRD config
    info!("Deactivating BIRD config for ASN {}", asn);
    bird::deploy::remove_config(asn)
        .map_err(|e| {
            error!("Failed to deactivate BIRD config for ASN {}: {}", asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deactivate BIRD config: {}", e))
        })?;

    info!("Successfully deactivated peering for ASN {} (config preserved in {})", asn, config_path);

    Ok(Json(UpdateResponse {
        status: "deactivated".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_request_deserialization() {
        let json = r#"{"asn": 4242420257}"#;
        let req: InitRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.asn, 4242420257);
    }

    #[test]
    fn test_init_response_serialization() {
        let resp = InitResponse {
            challenge: "AUTOPEER-4242420257-abc123".to_string(),
            pgp_fingerprint: "1234567890ABCDEF".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("AUTOPEER-4242420257-abc123"));
        assert!(json.contains("challenge"));
        assert!(json.contains("pgp_fingerprint"));
    }

    // Endpoint handler tests
    mod handler_tests {
        use super::*;
        use crate::api::test_helpers::test_config_with_temp_dirs;
        use axum::{body::Body, http::{Request, StatusCode}, Router, routing::post};
        use tower::ServiceExt;

        #[tokio::test]
        async fn test_init_peering_success() {
            let (config, _pending_dir, _verified_dir) = test_config_with_temp_dirs();
            let test_asn = 4242420257; // Use a real ASN with GPG key in registry

            let app = Router::new()
                .route("/peering/init", post(init_peering))
                .with_state(config.clone());

            let request_body = serde_json::to_string(&InitRequest { asn: test_asn }).unwrap();
            let request = Request::builder()
                .method("POST")
                .uri("/peering/init")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let init_response: InitResponse = serde_json::from_slice(&body).unwrap();

            // Check challenge format
            assert!(init_response.challenge.starts_with(&format!("AUTOPEER-{}-", test_asn)));

            // Check that PGP fingerprint was returned
            assert!(!init_response.pgp_fingerprint.is_empty());
            assert_eq!(init_response.pgp_fingerprint, "8B7F0384CBE0272761D852EA0684E36E6CF9D4D4");

            // Verify pending challenge file was created
            let iface_name = crate::ipalloc::interface_name(test_asn);
            let challenge_path = std::path::PathBuf::from(&config.data_pending_dir)
                .join(format!("{}.conf", iface_name));
            assert!(challenge_path.exists(), "Pending challenge file should exist at {:?}", challenge_path);

            // Verify challenge content matches
            let stored_challenge = std::fs::read_to_string(&challenge_path).unwrap();
            assert_eq!(stored_challenge, init_response.challenge);
        }

        #[tokio::test]
        async fn test_init_peering_invalid_asn() {
            let (config, _pending_dir, _verified_dir) = test_config_with_temp_dirs();

            let app = Router::new()
                .route("/peering/init", post(init_peering))
                .with_state(config);

            // Invalid ASN (too small for DN42)
            let request_body = serde_json::to_string(&InitRequest { asn: 1000 }).unwrap();
            let request = Request::builder()
                .method("POST")
                .uri("/peering/init")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();

            // Should fail with 400 Bad Request
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }


        #[tokio::test]
        async fn test_init_peering_creates_unique_challenges() {
            let (config, _pending_dir, _verified_dir) = test_config_with_temp_dirs();
            let test_asn = 4242420257; // Use a real ASN with GPG key in registry

            // Create first peering
            let app1 = Router::new()
                .route("/peering/init", post(init_peering))
                .with_state(config.clone());

            let request_body = serde_json::to_string(&InitRequest { asn: test_asn }).unwrap();
            let request1 = Request::builder()
                .method("POST")
                .uri("/peering/init")
                .header("content-type", "application/json")
                .body(Body::from(request_body.clone()))
                .unwrap();

            let response1 = app1.oneshot(request1).await.unwrap();
            let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX).await.unwrap();
            let resp1: InitResponse = serde_json::from_slice(&body1).unwrap();

            // Create second peering with same ASN
            let app2 = Router::new()
                .route("/peering/init", post(init_peering))
                .with_state(config);

            let request2 = Request::builder()
                .method("POST")
                .uri("/peering/init")
                .header("content-type", "application/json")
                .body(Body::from(request_body))
                .unwrap();

            let response2 = app2.oneshot(request2).await.unwrap();
            let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX).await.unwrap();
            let resp2: InitResponse = serde_json::from_slice(&body2).unwrap();

            // Challenges should be different (random)
            assert_ne!(resp1.challenge, resp2.challenge);
        }
    }
}
