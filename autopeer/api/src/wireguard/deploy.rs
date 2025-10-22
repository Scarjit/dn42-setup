use std::path::{Path, PathBuf};
use std::process::Command;

/// WireGuard keypair (private and public key)
#[derive(Debug, Clone)]
pub struct WgKeypair {
    pub private_key: String,
    pub public_key: String,
}

impl WgKeypair {
    /// Generate a new WireGuard keypair using wg command
    pub fn generate() -> Result<Self, String> {
        // Generate private key
        let private_output = Command::new("wg")
            .arg("genkey")
            .output()
            .map_err(|e| format!("Failed to run wg genkey: {}", e))?;

        if !private_output.status.success() {
            return Err(format!(
                "wg genkey failed: {}",
                String::from_utf8_lossy(&private_output.stderr)
            ));
        }

        let private_key = String::from_utf8_lossy(&private_output.stdout)
            .trim()
            .to_string();

        // Generate public key from private key
        let mut child = Command::new("wg")
            .arg("pubkey")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn wg pubkey: {}", e))?;

        // Write private key to stdin
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(private_key.as_bytes())
                .map_err(|e| format!("Failed to write to wg pubkey stdin: {}", e))?;
        }

        let public_output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait for wg pubkey: {}", e))?;

        if !public_output.status.success() {
            return Err(format!(
                "wg pubkey failed: {}",
                String::from_utf8_lossy(&public_output.stderr)
            ));
        }

        let public_key = String::from_utf8_lossy(&public_output.stdout)
            .trim()
            .to_string();

        Ok(WgKeypair {
            private_key,
            public_key,
        })
    }
}

/// Deploy a WireGuard configuration
pub fn deploy_config(config_content: &str, interface_name: &str) -> Result<(), String> {
    let config_path = PathBuf::from("/etc/wireguard").join(format!("{}.conf", interface_name));

    // Write config to /etc/wireguard/
    std::fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write config to {:?}: {}", config_path, e))?;

    // Bring up the interface using wg-quick
    let output = Command::new("wg-quick")
        .arg("up")
        .arg(interface_name)
        .output()
        .map_err(|e| format!("Failed to run wg-quick up: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "wg-quick up failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Remove a WireGuard configuration
pub fn remove_config(interface_name: &str) -> Result<(), String> {
    // Bring down the interface
    let output = Command::new("wg-quick")
        .arg("down")
        .arg(interface_name)
        .output()
        .map_err(|e| format!("Failed to run wg-quick down: {}", e))?;

    if !output.status.success() {
        // If it's not running, that's okay
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("is not a WireGuard interface") {
            return Err(format!("wg-quick down failed: {}", stderr));
        }
    }

    // Remove config file
    let config_path = PathBuf::from("/etc/wireguard").join(format!("{}.conf", interface_name));
    if config_path.exists() {
        std::fs::remove_file(&config_path)
            .map_err(|e| format!("Failed to remove config file {:?}: {}", config_path, e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = WgKeypair::generate().unwrap();

        // Keys should be base64-encoded and non-empty
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.public_key.is_empty());

        // Keys should be different
        assert_ne!(keypair.private_key, keypair.public_key);

        // Keys should be base64 (44 characters for WireGuard keys)
        assert_eq!(keypair.private_key.len(), 44);
        assert_eq!(keypair.public_key.len(), 44);
    }

    #[test]
    fn test_generate_multiple_keypairs() {
        let keypair1 = WgKeypair::generate().unwrap();
        let keypair2 = WgKeypair::generate().unwrap();

        // Each generation should produce unique keys
        assert_ne!(keypair1.private_key, keypair2.private_key);
        assert_ne!(keypair1.public_key, keypair2.public_key);
    }

    // Note: Deployment tests are skipped as they require root privileges
    // and would modify system configuration
}
