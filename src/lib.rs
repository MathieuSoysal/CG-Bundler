//! Rust Singler - A code compression tool for Rust
//! 
//! This library provides functionality to compress Rust source code by removing
//! comments, test code, and unnecessary whitespace while preserving semantic meaning.

pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod file_processor;
pub mod minifier;
pub mod parser;
pub mod performance;
pub mod reporter;
pub mod rust_singler;
pub mod syntax_tree;
pub mod traits;

pub use error::{ProcessingError, Result};
pub use config::CompressionConfig;
pub use rust_singler::RustSingler;
