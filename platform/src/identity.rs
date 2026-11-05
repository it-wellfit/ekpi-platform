// platform/src/identity.rs - Identity management shim
// Re-exports from crates/adapters for platform compatibility

pub use ekp_adapters::identity::{InMemoryIdentity, LoginThrottleStore, SessionStore, UserStore};
pub use ekp_adapters::password::Argon2idHasher;
pub use ekp_domain::auth::{Role, Session, UserRecord};
