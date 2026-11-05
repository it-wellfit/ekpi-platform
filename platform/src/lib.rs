#![allow(dead_code)]
#![forbid(unsafe_code)]

//! EKPI platform library.

pub mod agents;
pub mod api;
pub mod audit;
pub mod auth;
pub mod benchmark;
pub mod bpmn;
pub mod causal;
pub mod config;
pub mod coverage_report;
pub mod docs;
pub mod engine;
pub mod errors;
pub mod eval;
pub mod evidence;
pub mod export;
pub mod formal_verification;
pub mod fulltext;
pub mod governance;
pub mod governance_web;
pub mod identity;
pub mod knowledge_graph;
pub mod local_auth;
mod local_io;
mod local_lock;
pub mod multimodal;
pub mod ocr;
pub mod params;
pub mod pm4py_sidecar;
pub mod process;
pub mod rag;
pub mod ready;
pub mod reranker;
pub mod retrieval;
pub mod retrieval_v2;
pub mod rules;
pub mod runtime_verification;
pub mod session;
pub mod snapshot;
pub mod tenant;
pub mod token;
pub mod validate;
pub mod vector_search;
pub mod worm;
pub mod xes_import;
