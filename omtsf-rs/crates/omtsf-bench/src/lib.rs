//! Supply chain graph generator and benchmark utilities for OMTSF.
//!
//! This crate provides deterministic generation of realistic `.omts` files
//! for benchmarking and property-based testing of `omtsf-core`.

pub mod correctness;
pub mod generator;

pub use generator::{GeneratorConfig, SizeTier, generate_supply_chain};
