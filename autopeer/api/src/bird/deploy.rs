use std::path::PathBuf;
use std::process::Command;

/// Deploy a BIRD BGP peer configuration
pub fn deploy_config(config_content: &str, asn: u32) -> Result<(), String> {
    let config_path = PathBuf::from("/etc/bird/peers").join(format!("autopeer_as{}.conf", asn));

    // Ensure the peers directory exists
    let peers_dir = PathBuf::from("/etc/bird/peers");
    if !peers_dir.exists() {
        std::fs::create_dir_all(&peers_dir)
            .map_err(|e| format!("Failed to create peers directory {:?}: {}", peers_dir, e))?;
    }

    // Write config to /etc/bird/peers/
    std::fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write config to {:?}: {}", config_path, e))?;

    // Reload BIRD configuration using birdc configure
    let output = Command::new("birdc")
        .arg("configure")
        .output()
        .map_err(|e| format!("Failed to run birdc configure: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "birdc configure failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Remove a BIRD BGP peer configuration
pub fn remove_config(asn: u32) -> Result<(), String> {
    let config_path = PathBuf::from("/etc/bird/peers").join(format!("autopeer_as{}.conf", asn));

    // Remove config file if it exists
    if config_path.exists() {
        std::fs::remove_file(&config_path)
            .map_err(|e| format!("Failed to remove config file {:?}: {}", config_path, e))?;
    }

    // Reload BIRD configuration
    let output = Command::new("birdc")
        .arg("configure")
        .output()
        .map_err(|e| format!("Failed to run birdc configure: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "birdc configure failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path_format() {
        // Verify the expected path format
        let asn: u32 = 4242420257;
        let expected_path = PathBuf::from("/etc/bird/peers/autopeer_as4242420257.conf");
        let actual_path = PathBuf::from("/etc/bird/peers").join(format!("autopeer_as{}.conf", asn));

        assert_eq!(actual_path, expected_path);
    }

    // Note: Deployment tests are skipped as they require:
    // 1. Root privileges to write to /etc/bird/peers/
    // 2. BIRD to be installed and running
}
