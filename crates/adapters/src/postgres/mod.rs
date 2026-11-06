#![forbid(unsafe_code)]

//! PostgreSQL adapters.

pub mod approval;
pub mod channels;
pub mod governance;
pub mod identity;
pub mod pool;

pub use pool::{PoolSettings, connect, is_reachable, migrate};
