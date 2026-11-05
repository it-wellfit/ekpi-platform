#![forbid(unsafe_code)]

//! Canonical hashing helper shared by the governance hash chain.

use sha2::{Digest, Sha256};

/// Return the lowercase hexadecimal SHA-256 digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_matches_known_digest() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn digest_is_stable_for_equal_input() {
        assert_eq!(sha256_hex(b"ekpi"), sha256_hex(b"ekpi"));
        assert_ne!(sha256_hex(b"ekpi"), sha256_hex(b"ekpj"));
    }
}
