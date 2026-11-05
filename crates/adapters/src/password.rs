#![forbid(unsafe_code)]

//! Argon2id password hashing adapter.
//!
//! The encoded hash is self describing: it carries the algorithm, version and
//! cost parameters, so the work factor can be raised later without invalidating
//! existing accounts.

use argon2::password_hash::PasswordHasher;
use argon2::password_hash::PasswordVerifier;
use argon2::{Algorithm, Argon2, Params, Version};
use ekp_application::ports::PasswordHasher as AppPasswordHasher;
use password_hash::phc::PasswordHash;

/// OWASP minimum configuration for Argon2id: 19 MiB of memory, two
/// iterations and a single lane.
pub const OWASP_MEMORY_KIB: u32 = 19 * 1024;
pub const OWASP_ITERATIONS: u32 = 2;
pub const OWASP_PARALLELISM: u32 = 1;

/// Salt length in bytes.
pub const SALT_BYTES: usize = 16;

/// Argon2id password hasher.
#[derive(Debug, Clone)]
pub struct Argon2idHasher {
    params: Params,
}

impl Argon2idHasher {
    /// Build a hasher with explicit cost parameters.
    pub fn new(memory_kib: u32, iterations: u32, parallelism: u32) -> Result<Self, String> {
        if memory_kib == 0 || iterations == 0 || parallelism == 0 {
            return Err("Argon2 cost parameters must be greater than zero".to_string());
        }
        let params = Params::new(memory_kib, iterations, parallelism, None)
            .map_err(|error| format!("invalid Argon2 parameters: {error}"))?;
        Ok(Self { params })
    }

    /// Build the hasher with the OWASP minimum configuration.
    pub fn default_owasp() -> Self {
        Self::new(OWASP_MEMORY_KIB, OWASP_ITERATIONS, OWASP_PARALLELISM).expect("valid parameters")
    }

    fn engine(&self) -> Argon2<'_> {
        Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone())
    }
}

impl Default for Argon2idHasher {
    fn default() -> Self {
        Self::default_owasp()
    }
}

impl AppPasswordHasher for Argon2idHasher {
    fn hash(&self, password: &str) -> Result<String, String> {
        self.engine()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| format!("hash password: {error}"))
    }

    fn verify(&self, password: &str, encoded: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(encoded) else {
            return false;
        };
        self.engine()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_hasher() -> Argon2idHasher {
        Argon2idHasher::new(8, 1, 1).expect("valid parameters")
    }

    #[test]
    fn hash_round_trip_accepts_the_right_password() {
        let hasher = fast_hasher();
        let encoded = hasher.hash("correct horse battery").expect("hash succeeds");
        assert!(hasher.verify("correct horse battery", &encoded));
        assert!(!hasher.verify("wrong horse battery", &encoded));
    }

    #[test]
    fn hashes_are_salted_so_equal_passwords_differ() {
        let hasher = fast_hasher();
        let first = hasher.hash("same password").expect("hash succeeds");
        let second = hasher.hash("same password").expect("hash succeeds");
        assert_ne!(first, second);
        assert!(hasher.verify("same password", &first));
        assert!(hasher.verify("same password", &second));
    }

    #[test]
    fn plaintext_never_appears_in_the_encoded_hash() {
        let hasher = fast_hasher();
        let encoded = hasher.hash("plaintext-secret").expect("hash succeeds");
        assert!(!encoded.contains("plaintext-secret"));
        assert!(encoded.starts_with("$argon2id$"));
    }

    #[test]
    fn malformed_encoded_hashes_are_rejected() {
        let hasher = fast_hasher();
        assert!(!hasher.verify("password", "not-a-hash"));
        assert!(!hasher.verify("password", ""));
    }

    #[test]
    fn zero_cost_parameters_are_refused() {
        assert!(Argon2idHasher::new(0, 2, 1).is_err());
        assert!(Argon2idHasher::new(1024, 0, 1).is_err());
        assert!(Argon2idHasher::new(1024, 2, 0).is_err());
    }
}
