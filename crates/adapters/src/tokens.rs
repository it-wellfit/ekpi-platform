#![forbid(unsafe_code)]

//! Cryptographically secure random token adapter.

use ekp_application::ports::TokenGenerator;

/// Token generator backed by the operating system CSPRNG.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsTokenGenerator;

impl OsTokenGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TokenGenerator for OsTokenGenerator {
    fn generate(&self, bytes: usize) -> Result<String, String> {
        if bytes == 0 {
            return Err("token length must be greater than zero".to_string());
        }
        let mut buffer = vec![0u8; bytes];
        getrandom::getrandom(&mut buffer)
            .map_err(|error| format!("operating-system random source unavailable: {error}"))?;
        Ok(to_hex(&buffer))
    }
}

/// Lowercase hexadecimal encoding.
pub fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_have_the_requested_entropy_length() {
        let generator = OsTokenGenerator::new();
        let token = generator.generate(32).expect("token generated");
        assert_eq!(token.len(), 64);
        assert!(token.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn tokens_are_not_repeated() {
        let generator = OsTokenGenerator::new();
        let first = generator.generate(32).expect("token generated");
        let second = generator.generate(32).expect("token generated");
        assert_ne!(first, second);
    }

    #[test]
    fn zero_length_requests_are_refused() {
        assert!(OsTokenGenerator::new().generate(0).is_err());
    }
}
