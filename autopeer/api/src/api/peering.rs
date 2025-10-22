use crate::bird;
use crate::challenge::{gpg::verify_signature, Challenge};
use crate::config::AppConfig;
use crate::ipalloc::{interface_name, wireguard_port, Ipv6LinkLocal};
use crate::jwt::generate_token;
use crate::middleware::JwtAuth;
use crate::registry::{get_pgp_fingerprint_for_asn, verify_key_fingerprint};
use crate::validation;
use crate::wireguard::{self, BgpConfig, ChallengeConfig, InterfaceConfig, PeerConfig, WgConfig, WgKeypair};
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
    /// The peer's WireGuard public key
    pub peer_public_key: String,
    /// The skeleton WireGuard config for the peer
    pub wireguard_config: String,
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

    // Generate WireGuard keypair for this peer
    let keypair = WgKeypair::generate()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate keypair: {}", e)))?;

    // Allocate IPs
    let ips = Ipv6LinkLocal::from_asns(config.my_asn, req.asn);

    // Create skeleton WireGuard config with Challenge section
    let wg_config = WgConfig {
        interface: InterfaceConfig {
            address: vec![ips.peer.clone()],
            private_key: keypair.private_key.clone(),
            listen_port: wireguard_port(req.asn),
            table: Some("off".to_string()),
        },
        peer: None, // Will be filled in after verification
        challenge: Some(ChallengeConfig {
            code: challenge.code.clone(),
            asn: req.asn,
        }),
        bgp: None, // Will be filled in after verification
    };

    // Generate config string
    let config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate config: {}", e)))?;

    // Store the config somewhere (filesystem for now)
    let iface_name = interface_name(req.asn);
    let config_path = format!("{}/{}.conf", config.data_pending_dir, iface_name);

    // Ensure pending directory exists
    std::fs::create_dir_all(&config.data_pending_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create pending dir: {}", e)))?;

    wg_config
        .to_file(&config_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save config: {}", e)))?;

    Ok(Json(InitResponse {
        challenge: challenge.code,
        peer_public_key: keypair.public_key,
        wireguard_config: config_str,
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
    /// The peer's WireGuard public key (for the complete config)
    pub wg_public_key: String,
    /// The peer's public endpoint (IP:port)
    pub endpoint: String,
}

/// Response from peering verification
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyResponse {
    /// JWT token for authenticated operations
    pub token: String,
    /// The complete WireGuard config with peer and BGP sections
    pub wireguard_config: String,
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
    validation::validate_wg_pubkey(&req.wg_public_key)?;
    validation::validate_endpoint(&req.endpoint)?;
    validation::validate_pgp_key(&req.public_key)?;
    validation::validate_signed_challenge(&req.signed_challenge)?;

    // Load pending config
    let iface_name = interface_name(req.asn);
    let config_path = format!("{}/{}.conf", config.data_pending_dir, iface_name);

    let mut wg_config = WgConfig::from_file(&config_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Pending config not found: {}", e)))?;

    // Get challenge from config
    let challenge_config = wg_config
        .challenge
        .as_ref()
        .ok_or((StatusCode::BAD_REQUEST, "No challenge found in config".to_string()))?;

    // Verify ASN matches
    if challenge_config.asn != req.asn {
        return Err((StatusCode::BAD_REQUEST, "ASN mismatch".to_string()));
    }

    // Verify GPG signature
    let signature_valid = verify_signature(&challenge_config.code, &req.signed_challenge, &req.public_key)
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

    // Complete the WireGuard config
    let ips = Ipv6LinkLocal::from_asns(config.my_asn, req.asn);

    // Add peer section with endpoint from verify request
    wg_config.peer = Some(PeerConfig {
        public_key: req.wg_public_key.clone(),
        endpoint: Some(req.endpoint),
        allowed_ips: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
        persistent_keepalive: Some(25),
    });

    // Add BGP section
    wg_config.bgp = Some(BgpConfig {
        mpbgp: true,
        extended_next_hop: true,
        local: ips.local_addr(),
        neighbor: ips.peer.clone(),
    });

    // Remove challenge section (no longer needed)
    wg_config.challenge = None;

    // Generate complete config
    let complete_config = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate config: {}", e)))?;

    // Move config from pending to verified
    let verified_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);
    std::fs::create_dir_all(&config.data_verified_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create verified dir: {}", e)))?;

    wg_config
        .to_file(&verified_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save verified config: {}", e)))?;

    // Remove pending config
    let _ = std::fs::remove_file(&config_path);

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
        wireguard_config: complete_config,
    }))
}

/// Request to deploy a peering
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployRequest {
    /// The peer's ASN
    pub asn: u32,
}

/// Response from peering deployment
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployResponse {
    /// Status message
    pub status: String,
    /// The deployed interface name
    pub interface: String,
}

/// POST /peering/deploy - Deploy a verified peering configuration
pub async fn deploy_peering(
    State(config): State<Arc<AppConfig>>,
    auth: JwtAuth,
    Json(req): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, (StatusCode, String)> {
    let asn = auth.asn;
    info!("Peering deploy request for ASN {}", asn);

    // Validate ASN matches request
    if asn != req.asn {
        return Err((StatusCode::BAD_REQUEST, "ASN mismatch between JWT and request".to_string()));
    }

    // Load verified config
    let iface_name = interface_name(req.asn);
    let config_path = format!("{}/{}.conf", config.data_verified_dir, iface_name);

    let wg_config = WgConfig::from_file(&config_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Verified config not found: {}", e)))?;

    // Generate WireGuard config string
    let wg_config_str = wg_config
        .as_string()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate WireGuard config: {}", e)))?;

    // Deploy WireGuard configuration
    info!("Deploying WireGuard config for ASN {} ({})", req.asn, iface_name);
    wireguard::deploy::deploy_config(&wg_config_str, &iface_name)
        .map_err(|e| {
            error!("Failed to deploy WireGuard for ASN {}: {}", req.asn, e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deploy WireGuard: {}", e))
        })?;

    // Generate and deploy BIRD configuration if BGP config exists
    if wg_config.bgp.is_some() {
        info!("Deploying BIRD config for ASN {}", req.asn);

        let bird_peer_config = bird::BirdPeerConfig::new(
            config.my_asn,
            req.asn,
            format!("AS{}", req.asn),
            iface_name.clone(),
        );

        let bird_config_str = bird_peer_config
            .to_config()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate BIRD config: {}", e)))?;

        bird::deploy::deploy_config(&bird_config_str, req.asn)
            .map_err(|e| {
                error!("Failed to deploy BIRD config for ASN {}: {}", req.asn, e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to deploy BIRD config: {}", e))
            })?;
    }

    info!("Successfully deployed peering for ASN {}", req.asn);

    Ok(Json(DeployResponse {
        status: "deployed".to_string(),
        interface: iface_name,
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
            peer_public_key: "testpubkey123==".to_string(),
            wireguard_config: "[Interface]\nPrivateKey=test".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("AUTOPEER-4242420257-abc123"));
        assert!(json.contains("testpubkey123=="));
    }
}
