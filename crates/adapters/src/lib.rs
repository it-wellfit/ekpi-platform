#![forbid(unsafe_code)]

//! EKPI infrastructure adapters.
//!
//! Every port declared by the application layer has at least one adapter in
//! this crate. Adapters own the storage technology: JSONL files for the local
//! pilot, PostgreSQL for the deployment target, Argon2id for password hashing
//! and the operating system for randomness.

pub mod audit;
pub mod configuration;
pub mod identity;
pub mod jsonl_ledger;
pub mod local_io;
pub mod local_lock;
pub mod password;
pub mod postgres;
pub mod tokens;
