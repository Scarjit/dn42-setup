use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents an AS object from the DN42 registry
#[derive(Debug, Clone, PartialEq)]
pub struct AsObject {
    pub asn: u32,
    pub as_name: String,
    pub description: String,
    pub admin_c: String,
    pub tech_c: String,
    pub mnt_by: String,
}

/// Represents a maintainer object
#[derive(Debug, Clone, PartialEq)]
pub struct MaintainerObject {
    pub mntner: String,
    pub description: String,
    pub auth_fingerprints: Vec<String>,
}

/// Represents a PGP key certificate
#[derive(Debug, Clone, PartialEq)]
pub struct KeyCert {
    pub key_id: String,
    pub method: String,
    pub fingerprint: String,
    pub owner: String,
    pub public_key: String,
}

/// Parse a DN42 registry object file (generic key-value format)
fn parse_registry_object(content: &str) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key: value
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();

            result.entry(key).or_default().push(value);
        }
    }

    result
}

/// Parse an AS object
pub fn parse_as_object(content: &str) -> Result<AsObject, String> {
    let fields = parse_registry_object(content);

    let asn_str = fields
        .get("aut-num")
        .and_then(|v| v.first())
        .ok_or("Missing aut-num field")?;

    // Parse ASN (format: AS4242420257)
    let asn = asn_str
        .strip_prefix("AS")
        .ok_or("Invalid ASN format")?
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse ASN: {}", e))?;

    let as_name = fields
        .get("as-name")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    let description = fields
        .get("descr")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    let admin_c = fields
        .get("admin-c")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    let tech_c = fields
        .get("tech-c")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    let mnt_by = fields
        .get("mnt-by")
        .and_then(|v| v.first())
        .ok_or("Missing mnt-by field")?
        .clone();

    Ok(AsObject {
        asn,
        as_name,
        description,
        admin_c,
        tech_c,
        mnt_by,
    })
}

/// Parse a maintainer object
pub fn parse_maintainer(content: &str) -> Result<MaintainerObject, String> {
    let fields = parse_registry_object(content);

    let mntner = fields
        .get("mntner")
        .and_then(|v| v.first())
        .ok_or("Missing mntner field")?
        .clone();

    let description = fields
        .get("descr")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    // Extract PGP fingerprints from auth fields
    let mut auth_fingerprints = Vec::new();
    if let Some(auth_values) = fields.get("auth") {
        for auth in auth_values {
            if let Some(fingerprint) = auth.strip_prefix("pgp-fingerprint ") {
                auth_fingerprints.push(fingerprint.to_string());
            }
        }
    }

    Ok(MaintainerObject {
        mntner,
        description,
        auth_fingerprints,
    })
}

/// Parse a key-cert object
pub fn parse_key_cert(content: &str) -> Result<KeyCert, String> {
    let fields = parse_registry_object(content);

    let key_id = fields
        .get("key-cert")
        .and_then(|v| v.first())
        .ok_or("Missing key-cert field")?
        .clone();

    let method = fields
        .get("method")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| "PGP".to_string());

    let fingerprint = fields
        .get("fingerpr")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    let owner = fields
        .get("owner")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    // Reconstruct PGP public key from certif lines
    let mut public_key = String::new();
    if let Some(certif_lines) = fields.get("certif") {
        for line in certif_lines {
            public_key.push_str(line);
            public_key.push('\n');
        }
    }

    Ok(KeyCert {
        key_id,
        method,
        fingerprint,
        owner,
        public_key,
    })
}

/// Get AS object from registry by ASN
pub fn get_as_object<P: AsRef<Path>>(registry_path: P, asn: u32) -> Result<AsObject, String> {
    let as_file = registry_path
        .as_ref()
        .join("data/aut-num")
        .join(format!("AS{}", asn));

    let content =
        fs::read_to_string(&as_file).map_err(|e| format!("Failed to read AS{}: {}", asn, e))?;

    parse_as_object(&content)
}

/// Get maintainer object from registry
pub fn get_maintainer<P: AsRef<Path>>(
    registry_path: P,
    mntner: &str,
) -> Result<MaintainerObject, String> {
    let mnt_file = registry_path.as_ref().join("data/mntner").join(mntner);

    let content =
        fs::read_to_string(&mnt_file).map_err(|e| format!("Failed to read {}: {}", mntner, e))?;

    parse_maintainer(&content)
}

/// Get PGP fingerprint for an ASN from the registry
pub fn get_pgp_fingerprint_for_asn<P: AsRef<Path>>(
    registry_path: P,
    asn: u32,
) -> Result<String, String> {
    // 1. Get AS object
    let as_obj = get_as_object(&registry_path, asn)?;

    // 2. Get maintainer
    let mnt = get_maintainer(&registry_path, &as_obj.mnt_by)?;

    // 3. Get first PGP fingerprint
    let fingerprint = mnt
        .auth_fingerprints
        .first()
        .ok_or("No PGP fingerprint found for maintainer")?
        .clone();

    Ok(fingerprint)
}

/// Verify that a public key matches the expected fingerprint
pub fn verify_key_fingerprint(
    public_key: &str,
    expected_fingerprint: &str,
) -> Result<bool, String> {
    use pgp::composed::{Deserializable, SignedPublicKey};
    use pgp::types::KeyDetails;

    // Parse the public key
    let (key, _) = SignedPublicKey::from_string(public_key)
        .map_err(|e| format!("Failed to parse public key: {}", e))?;

    // Get the fingerprint from the key
    let key_fingerprint = format!("{:X}", key.fingerprint());

    // Normalize both fingerprints (remove spaces, uppercase)
    let normalized_expected = expected_fingerprint.replace(' ', "").to_uppercase();
    let normalized_key = key_fingerprint.replace(' ', "").to_uppercase();

    Ok(normalized_key == normalized_expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_as_object() {
        let content = r#"
aut-num:            AS4242420257
as-name:            SCARJIT-AS
descr:              SCARJIT Network
admin-c:            SCARJIT-DN42
tech-c:             SCARJIT-DN42
mnt-by:             SCARJIT-MNT
source:             DN42
"#;

        let as_obj = parse_as_object(content).unwrap();
        assert_eq!(as_obj.asn, 4242420257);
        assert_eq!(as_obj.as_name, "SCARJIT-AS");
        assert_eq!(as_obj.mnt_by, "SCARJIT-MNT");
    }

    #[test]
    fn test_parse_maintainer() {
        let content = r#"
mntner:             SCARJIT-MNT
descr:              SCARJIT https://linnenberg.dev/
admin-c:            SCARJIT-DN42
tech-c:             SCARJIT-DN42
auth:               pgp-fingerprint 8B7F0384CBE0272761D852EA0684E36E6CF9D4D4
mnt-by:             SCARJIT-MNT
source:             DN42
"#;

        let mnt = parse_maintainer(content).unwrap();
        assert_eq!(mnt.mntner, "SCARJIT-MNT");
        assert_eq!(mnt.auth_fingerprints.len(), 1);
        assert_eq!(
            mnt.auth_fingerprints[0],
            "8B7F0384CBE0272761D852EA0684E36E6CF9D4D4"
        );
    }

    #[test]
    fn test_get_as_object_from_registry() {
        dotenvy::dotenv().ok();
        let registry_path = std::env::var("DN42_REGISTRY_PATH")
            .unwrap_or_else(|_| "./data/dn42-registry".to_string());

        let as_obj = get_as_object(&registry_path, 4242420257).unwrap();
        assert_eq!(as_obj.asn, 4242420257);
        assert_eq!(as_obj.as_name, "SCARJIT-AS");
    }

    #[test]
    fn test_get_pgp_fingerprint_and_verify() {
        dotenvy::dotenv().ok();
        let registry_path = std::env::var("DN42_REGISTRY_PATH")
            .unwrap_or_else(|_| "./data/dn42-registry".to_string());

        // Get fingerprint from registry
        let fingerprint = get_pgp_fingerprint_for_asn(&registry_path, 4242420257).unwrap();
        assert_eq!(fingerprint, "8B7F0384CBE0272761D852EA0684E36E6CF9D4D4");

        // Verify that your actual public key matches
        let public_key = r#"-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEYVuS5RYJKwYBBAHaRw8BAQdAnJ1to/QytFqDfg3gtUrtiqmJRMSLNrG/fLNG
BesjX5m0L0ZlcmRpbmFuZCBMaW5uZW5iZXJnIDxmZXJkaW5hbmRAbGlubmVuYmVy
Zy5kZXY+iJAEExYIADgWIQSLfwOEy+AnJ2HYUuoGhONubPnU1AUCYVuS5QIbAwUL
CQgHAgYVCgkICwIEFgIDAQIeAQIXgAAKCRAGhONubPnU1M2ZAP0drb1tbnLi1cU+
Pc4NPTMjviTBBFmGFoDni/0mvMC5qAD6AlB24idciDkSeJFz3s/6wSog/Rj4ALpk
RQ/v8Ls4gQa4OARhW5LlEgorBgEEAZdVAQUBAQdAci4cwabJdJGO+VF5wxEW+yuO
Y+BPprEQpy4jFiN713sDAQgHiHgEGBYIACAWIQSLfwOEy+AnJ2HYUuoGhONubPnU
1AUCYVuS5QIbDAAKCRAGhONubPnU1I79AQC7Weudp5yzofVqZQCa/ijohC5CuwXw
LGZbH16nUawo9gEAw+6wvpgw2d7IS6rnT6jJZ1qm6inF/XzTZTNfq9rsmgM=
=WrLZ
-----END PGP PUBLIC KEY BLOCK-----"#;

        let is_valid = verify_key_fingerprint(public_key, &fingerprint).unwrap();
        assert!(is_valid, "Public key fingerprint should match");
    }
}
