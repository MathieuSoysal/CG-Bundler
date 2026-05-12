//! Test fixtures shared across the binary's command modules.
//!
//! Compiled only under `cfg(test)`, this avoids duplicating the small
//! Cargo project layouts used as integration scaffolding.

use std::fs;
use std::path::Path;

const MINIMAL_CARGO_TOML: &str = r#"[package]
name = "test_proj"
version = "0.1.0"
edition = "2021"
"#;

const CARGO_TOML_WITH_LIB: &str = r#"[package]
name = "test_proj"
version = "0.1.0"
edition = "2021"
description = "a test project"

[[bin]]
name = "test_proj"
path = "src/main.rs"

[lib]
name = "test_proj"
path = "src/lib.rs"
"#;

/// Create a minimal binary-only Cargo project at `dir` with the given `main.rs` body.
pub fn make_project(dir: &Path, main_src: &str) {
    fs::write(dir.join("Cargo.toml"), MINIMAL_CARGO_TOML).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), main_src).unwrap();
}

/// Create a Cargo project at `dir` with both a binary and a library target.
pub fn make_project_with_lib(dir: &Path) {
    fs::write(dir.join("Cargo.toml"), CARGO_TOML_WITH_LIB).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("src/lib.rs"),
        "pub fn greet() -> &'static str { \"hi\" }",
    )
    .unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
}
