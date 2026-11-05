// platform/src/auth.rs - Authentication shim
// Re-exports from crates/adapters

pub use ekp_adapters::identity::InMemoryIdentity;
pub use ekp_adapters::password::Argon2idHasher;
