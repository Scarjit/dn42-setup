pub mod gpg;

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A challenge code for peer authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Challenge {
    pub code: String,
    pub asn: u32,
}

impl Challenge {
    /// Generate a new random challenge for the given ASN
    pub fn generate(asn: u32) -> Self {
        let mut rng = rand::rng();
        let random_bytes: [u8; 16] = rng.random();
        let code = format!("AUTOPEER-{}-{}", asn, hex::encode(random_bytes));

        Challenge { code, asn }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge() {
        let asn = 4242421234;
        let challenge = Challenge::generate(asn);

        assert_eq!(challenge.asn, asn);
        assert!(challenge.code.starts_with("AUTOPEER-"));
        assert!(challenge.code.contains(&asn.to_string()));
    }

    #[test]
    fn test_challenges_are_unique() {
        let asn = 4242421234;
        let challenge1 = Challenge::generate(asn);
        let challenge2 = Challenge::generate(asn);

        assert_ne!(challenge1.code, challenge2.code);
    }

    #[test]
    fn test_challenge_serialization() {
        let challenge = Challenge {
            code: "AUTOPEER-4242421234-0123456789abcdef0123456789abcdef".to_string(),
            asn: 4242421234,
        };

        let json = serde_json::to_string(&challenge).unwrap();
        let deserialized: Challenge = serde_json::from_str(&json).unwrap();

        assert_eq!(challenge, deserialized);
    }

}
