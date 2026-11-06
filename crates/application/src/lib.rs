#![forbid(unsafe_code)]

//! EKPI application layer.
//!
//! Use cases live here. They orchestrate the domain rules and talk to the
//! outside world only through the ports declared in [`ports`], so the same
//! use cases can run against the local JSONL pilot store and against the
//! PostgreSQL deployment.

pub mod approval;
pub mod authentication;
pub mod authorization;
pub mod governance;
pub mod ports;
