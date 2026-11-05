#![forbid(unsafe_code)]

//! EKPI domain layer.
//!
//! This crate holds the business model and the business rules that do not
//! need a database, a filesystem, a clock or an HTTP framework. It must not
//! depend on Axum, Askama, SQLx or Tower: the dependency direction of the
//! workspace is Presentation -> Application -> Domain.

pub mod auth;
pub mod governance;
pub mod hashing;
pub mod tenant;
