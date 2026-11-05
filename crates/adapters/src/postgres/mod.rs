#![forbid(unsafe_code)]

//! PostgreSQL adapters.

pub mod governance;
pub mod identity;
pub mod pool;

pub use pool::{PoolSettings, connect, is_reachable, migrate};
