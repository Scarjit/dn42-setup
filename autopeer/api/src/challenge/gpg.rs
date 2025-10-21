use pgp::composed::{Deserializable, DetachedSignature, Message, SignedPublicKey};

/// Verify a GPG signature for the given message
/// Returns Ok(true) if signature is valid, Ok(false) if invalid, Err on failure
pub fn verify_signature(
    message: &str,
    signature: &str,
    public_key_str: &str,
) -> Result<bool, String> {
    // Parse the public key
    let (public_key, _headers) = SignedPublicKey::from_string(public_key_str)
        .map_err(|e| format!("Failed to parse public key: {}", e))?;

    // Try to parse as a cleartext signed message first
    if signature.contains("BEGIN PGP SIGNED MESSAGE") {
        // Extract the actual message and signature from cleartext format
        let parts: Vec<&str> = signature.split("-----BEGIN PGP SIGNATURE-----").collect();
        if parts.len() != 2 {
            return Err("Invalid cleartext signature format".to_string());
        }

        // Extract message from cleartext section
        let cleartext_section = parts[0];
        let lines: Vec<&str> = cleartext_section.lines().collect();

        // Find where the message starts (after the Hash: line and empty line)
        let mut message_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("Hash:") || line.starts_with("-----BEGIN PGP SIGNED MESSAGE-----") {
                // Message starts after next empty line
                if i + 1 < lines.len() && lines[i + 1].is_empty() {
                    message_start = i + 2;
                    break;
                }
            }
        }

        let msg_lines: Vec<&str> = lines
            .iter()
            .skip(message_start)
            .take_while(|line| !line.is_empty())
            .copied()
            .collect();

        let extracted_message = msg_lines.join("\n");

        // Build the signature block
        let sig_block = format!("-----BEGIN PGP SIGNATURE-----{}", parts[1]);

        // Parse the detached signature
        let (sig, _) = DetachedSignature::from_string(&sig_block)
            .map_err(|e| format!("Failed to parse signature: {}", e))?;

        // For cleartext signatures, PGP uses canonical text mode
        // The message needs to have a trailing newline
        let mut canonical_message = extracted_message.clone();
        if !canonical_message.ends_with('\n') {
            canonical_message.push('\n');
        }
        // Convert to CRLF for canonical text mode
        let canonical_message = canonical_message.replace('\n', "\r\n");

        // Verify the signature
        sig.verify(&public_key, canonical_message.as_bytes())
            .map_err(|e| format!("Signature verification failed: {}", e))?;

        // Check that the message content matches
        if extracted_message.trim() == message.trim() {
            Ok(true)
        } else {
            Err(format!(
                "Message content does not match. Expected: '{}', Got: '{}'",
                message.trim(),
                extracted_message.trim()
            ))
        }
    } else {
        // Try parsing as a regular message
        let (signed_msg, _headers) = Message::from_string(signature)
            .map_err(|e| format!("Failed to parse signature: {}", e))?;

        // Verify the signature
        signed_msg
            .verify(&public_key)
            .map_err(|e| format!("Signature verification failed: {}", e))?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature_invalid_key() {
        let message = "test message";
        let signature = "fake signature";
        let public_key = "not a valid key";

        let result = verify_signature(message, signature, public_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_real_signature() {
        let message = "AUTOPEER-AS4242420257-THISISATEST";

        let signature = r#"-----BEGIN PGP SIGNED MESSAGE-----
Hash: SHA512

AUTOPEER-AS4242420257-THISISATEST

-----BEGIN PGP SIGNATURE-----

iHUEARYKAB0WIQSLfwOEy+AnJ2HYUuoGhONubPnU1AUCaPe23gAKCRAGhONubPnU
1G50AP0bnfUm+rT/lag4MFTWuaYdD7kEIa/KjJ0hOwkX5yeFlwEAqzUAznyJ3dlI
5tsRBC4VYY8aBXfA8RycPLsPLy3WZws=
=Vr9+
-----END PGP SIGNATURE-----"#;

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

        let result = verify_signature(message, signature, public_key);
        assert!(result.is_ok(), "Verification failed: {:?}", result);
        assert!(result.unwrap(), "Signature should be valid");
    }

    #[test]
    fn test_verify_signature_wrong_message() {
        let message = "AUTOPEER-AS4242420257-WRONGMESSAGE";

        let signature = r#"-----BEGIN PGP SIGNED MESSAGE-----
Hash: SHA512

AUTOPEER-AS4242420257-THISISATEST

-----BEGIN PGP SIGNATURE-----

iHUEARYKAB0WIQSLfwOEy+AnJ2HYUuoGhONubPnU1AUCaPe23gAKCRAGhONubPnU
1G50AP0bnfUm+rT/lag4MFTWuaYdD7kEIa/KjJ0hOwkX5yeFlwEAqzUAznyJ3dlI
5tsRBC4VYY8aBXfA8RycPLsPLy3WZws=
=Vr9+
-----END PGP SIGNATURE-----"#;

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

        let result = verify_signature(message, signature, public_key);
        assert!(result.is_err(), "Should fail when message doesn't match");
    }
}
