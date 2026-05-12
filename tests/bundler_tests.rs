use cg_bundler::Bundler;
use cg_bundler::cargo_project::CargoProject;
use cg_bundler::error::BundlerError;
use cg_bundler::transformer::TransformConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn make_project(dir: &Path, main_src: &str) {
    let toml = r#"[package]
name = "bundler_test"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(dir.join("Cargo.toml"), toml).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), main_src).unwrap();
}

// ── Bundler::new / default / with_config ──────────────────────────────

#[test]
fn test_bundler_new_and_default_are_equivalent() {
    let b1 = Bundler::new();
    let b2 = Bundler::default();
    let c1 = b1.config();
    let c2 = b2.config();
    assert_eq!(c1.remove_tests, c2.remove_tests);
    assert_eq!(c1.remove_docs, c2.remove_docs);
    assert_eq!(c1.expand_modules, c2.expand_modules);
}

#[test]
fn test_with_config_and_set_config() {
    let custom = TransformConfig {
        remove_tests: false,
        remove_docs: false,
        expand_modules: false,
        minify: true,
        aggressive_minify: false,
    };
    let b = Bundler::with_config(custom.clone());
    assert!(!b.config().remove_tests);
    assert!(b.config().minify);

    let mut b2 = Bundler::new();
    b2.set_config(custom);
    assert!(!b2.config().remove_tests);
    assert!(b2.config().minify);
}

// ── bundle ─────────────────────────────────────────────────────────────

#[test]
fn test_bundle_simple_project() {
    let tmp = TempDir::new().unwrap();
    make_project(tmp.path(), "fn main() { println!(\"hi\"); }");

    let bundler = Bundler::new();
    let result = bundler.bundle(tmp.path());
    assert!(result.is_ok(), "bundle should succeed: {result:?}");
    let code = result.unwrap();
    assert!(code.contains("fn main"));
}

#[test]
fn test_bundle_nonexistent_path_returns_error() {
    let tmp = TempDir::new().unwrap();
    let bundler = Bundler::new();
    assert!(bundler.bundle(tmp.path().join("nope")).is_err());
}

// ── bundle_project ─────────────────────────────────────────────────────

#[test]
fn test_bundle_project_with_lib() {
    let tmp = TempDir::new().unwrap();
    let toml = r#"[package]
name = "bp_test"
version = "0.1.0"
edition = "2021"
[[bin]]
name = "bp_test"
path = "src/main.rs"
[lib]
name = "bp_test"
path = "src/lib.rs"
"#;
    fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(
        tmp.path().join("src/lib.rs"),
        "pub fn greet() -> &'static str { \"hi\" }",
    )
    .unwrap();
    fs::write(
        tmp.path().join("src/main.rs"),
        "fn main() { println!(\"{}\", bp_test::greet()); }",
    )
    .unwrap();

    let project = CargoProject::new(tmp.path()).unwrap();
    let bundler = Bundler::new();
    let result = bundler.bundle_project(&project);
    assert!(result.is_ok(), "bundle_project should succeed: {result:?}");
    assert!(result.unwrap().contains("fn main"));
}

#[test]
fn test_bundle_project_invalid_rust_source() {
    let tmp = TempDir::new().unwrap();
    let toml = "[package]\nname = \"inv_test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    fs::write(tmp.path().join("Cargo.toml"), toml).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    // Write syntactically invalid Rust
    fs::write(tmp.path().join("src/main.rs"), "fn main( { invalid syntax").unwrap();

    let project = CargoProject::new(tmp.path()).unwrap();
    let bundler = Bundler::new();
    let result = bundler.bundle_project(&project);
    assert!(result.is_err());
    match result.unwrap_err() {
        BundlerError::Parsing { .. } => {}
        e => panic!("Expected Parsing error, got: {e}"),
    }
}

// ── bundle convenience function (from lib.rs) ─────────────────────────

#[test]
fn test_bundle_convenience_fn_success() {
    let tmp = TempDir::new().unwrap();
    let cargo_toml = "[package]\nname = \"conv_test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    fs::write(tmp.path().join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();
    let result = cg_bundler::bundle(tmp.path());
    assert!(result.is_ok());
    assert!(result.unwrap().contains("fn main"));
}

#[test]
fn test_bundle_convenience_fn_error() {
    let result = cg_bundler::bundle("/nonexistent/path");
    assert!(result.is_err());
}
