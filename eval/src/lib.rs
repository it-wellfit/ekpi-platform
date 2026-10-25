#![forbid(unsafe_code)]

//! EKPI 評測 harness（規範性 Rust 實作）。
//!
//! 可重用模組：[`types`]（案例/決定型別與載入）、[`metrics`]（指標與硬門檻）、
//! [`report`]（JSON+Markdown 報告）、[`systems`]（adapter trait 與確定性 stub）。
//!
//! 外部系統（如 `platform` crate 的確定性引擎）以 [`systems::System`] trait
//! 接入，跑同一 runner 即得可比較的消融報告（addendum A7）。

pub mod annotation;
pub mod benchmark;
pub mod failure_cases;
pub mod metrics;
pub mod report;
pub mod systems;
pub mod types;
