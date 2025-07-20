use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::bundler::Bundler;
use crate::cargo_project::CargoProject;
use crate::transformer::TransformConfig;

/// Integration tests for the complete bundling process
#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_project_with_modules(temp_dir: &std::path::Path, name: &str) -> PathBuf {
        let project_path = temp_dir.join(name);
        fs::create_dir_all(&project_path).unwrap();

        // Create Cargo.toml
        let cargo_toml = format!(
            r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
path = "src/main.rs"

[lib]
name = "{name}"
path = "src/lib.rs"
"#
        );

        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_path.join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create main.rs with extern crate
        let main_content = format!(
            r#"
extern crate {name};

use {name}::greet;

fn main() {{
    println!("{{}}", greet("World"));
}}
"#
        );
        fs::write(src_dir.join("main.rs"), main_content).unwrap();

        // Create lib.rs with module
        let lib_content = r#"
pub mod utils;

/// Greet someone
pub fn greet(name: &str) -> String {
    utils::format_greeting(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("Test"), "Hello, Test!");
    }
}
"#;
        fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

        // Create utils.rs module
        let utils_content = r#"
/// Format a greeting message
/// This function creates a formatted greeting
pub fn format_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Helper function for testing
#[cfg(test)]
pub fn test_helper() -> &'static str {
    "test helper"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_greeting() {
        assert_eq!(format_greeting("Test"), "Hello, Test!");
    }
}
"#;
        fs::write(src_dir.join("utils.rs"), utils_content).unwrap();

        project_path
    }

    #[test]
    fn test_complete_bundling_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project_with_modules(temp_dir.path(), "test_workflow");

        // Test project creation
        let project = CargoProject::new(&project_path).unwrap();
        assert_eq!(project.crate_name(), "test_workflow");
        assert!(project.library_target().is_some());

        // Test bundling
        let bundler = Bundler::new();
        let result = bundler.bundle_project(&project).unwrap();

        // Verify bundled code contains expected elements
        assert!(result.contains("fn main"));
        assert!(result.contains("pub fn greet"));
        assert!(result.contains("pub fn format_greeting"));

        // Verify test code is removed
        assert!(!result.contains("#[test]"));
        assert!(!result.contains("test_helper"));
        assert!(!result.contains("mod tests"));

        // Verify documentation is removed
        assert!(!result.contains("/// Greet someone"));
        assert!(!result.contains("/// Format a greeting message"));
        assert!(!result.contains("/// This function creates"));

        // Verify the extern crate is expanded
        assert!(!result.contains("extern crate test_workflow"));

        // Verify code is syntactically valid
        syn::parse_file(&result).unwrap();
    }

    #[test]
    fn test_bundling_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project_with_modules(temp_dir.path(), "test_custom");

        let config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: true,
        };

        let bundler = Bundler::with_config(config);
        let result = bundler.bundle(&project_path).unwrap();

        // With custom config, tests and docs should be preserved
        assert!(result.contains("# [test]") || result.contains("#[test]"));
        assert!(
            result.contains("/// Greet someone") || result.contains("# [doc = \" Greet someone\"]")
        );
        assert!(result.contains("mod tests"));

        // Modules should still be expanded
        assert!(result.contains("pub fn format_greeting"));

        // Code should still be valid
        syn::parse_file(&result).unwrap();
    }

    #[test]
    fn test_bundling_preserves_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project_with_modules(temp_dir.path(), "test_preserve");

        let bundler = Bundler::new();
        let bundled_code = bundler.bundle(&project_path).unwrap();

        // Parse and verify the structure
        let parsed = syn::parse_file(&bundled_code).unwrap();

        // Count important items
        let mut main_count = 0;
        let mut greet_count = 0;
        let mut format_greeting_count = 0;

        for item in &parsed.items {
            match item {
                syn::Item::Fn(func) => {
                    let func_name = func.sig.ident.to_string();
                    match func_name.as_str() {
                        "main" => main_count += 1,
                        "greet" => greet_count += 1,
                        "format_greeting" => format_greeting_count += 1,
                        _ => {}
                    }
                }
                syn::Item::Mod(mod_item) => {
                    if let Some((_, items)) = &mod_item.content {
                        for item in items {
                            if let syn::Item::Fn(func) = item {
                                if func.sig.ident == "format_greeting" {
                                    format_greeting_count += 1;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        assert_eq!(main_count, 1, "Should have exactly one main function");
        assert_eq!(greet_count, 1, "Should have exactly one greet function");
        assert_eq!(
            format_greeting_count, 1,
            "Should have exactly one format_greeting function"
        );
    }

    #[test]
    fn test_error_handling_for_invalid_projects() {
        let temp_dir = TempDir::new().unwrap();

        // Test with non-existent project
        let result = Bundler::new().bundle(temp_dir.path().join("nonexistent"));
        assert!(result.is_err());

        // Test with invalid Cargo.toml
        let invalid_project = temp_dir.path().join("invalid");
        fs::create_dir_all(&invalid_project).unwrap();
        fs::write(invalid_project.join("Cargo.toml"), "invalid toml content").unwrap();

        let result = Bundler::new().bundle(&invalid_project);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_without_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("simple");
        fs::create_dir_all(&project_path).unwrap();

        let cargo_toml = r#"
[package]
name = "simple"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "simple"
path = "src/main.rs"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_path.join("src");
        fs::create_dir(&src_dir).unwrap();

        let main_content = r#"
fn main() {
    println!("Hello, World!");
}
"#;
        fs::write(src_dir.join("main.rs"), main_content).unwrap();

        let bundler = Bundler::new();
        let result = bundler.bundle(&project_path).unwrap();

        assert!(result.contains("fn main"));
        assert!(result.contains("Hello, World!"));

        // Verify it's valid Rust
        syn::parse_file(&result).unwrap();
    }

    #[test]
    fn test_bundler_config_mutations() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project_with_modules(temp_dir.path(), "test_config");

        let mut bundler = Bundler::new();

        // Test with default config
        let result1 = bundler.bundle(&project_path).unwrap();
        assert!(!result1.contains("#[test]"));
        assert!(!result1.contains("/// Greet"));

        // Change config to preserve docs and tests
        let new_config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: true,
        };
        bundler.set_config(new_config);

        let result2 = bundler.bundle(&project_path).unwrap();
        assert!(result2.contains("# [test]") || result2.contains("#[test]"));
        assert!(result2.contains("/// Greet") || result2.contains("# [doc"));

        // Verify the config is actually changed
        assert!(!bundler.config().remove_tests);
        assert!(!bundler.config().remove_docs);
        assert!(bundler.config().expand_modules);
    }

    #[test]
    fn test_large_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("large_project");
        fs::create_dir_all(&project_path).unwrap();

        // Create a more complex project structure
        let cargo_toml = r#"
[package]
name = "large_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "large_project"
path = "src/main.rs"

[lib]
name = "large_project"
path = "src/lib.rs"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_path.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create nested module structure
        let core_dir = src_dir.join("core");
        fs::create_dir_all(&core_dir).unwrap();

        // Main file
        fs::write(
            src_dir.join("main.rs"),
            r#"
extern crate large_project;

use large_project::core::engine::run;

fn main() {
    run();
}
"#,
        )
        .unwrap();

        // Lib file
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod core;
pub mod utils;

pub fn init() {
    println!("Initializing...");
}
"#,
        )
        .unwrap();

        // Core module
        fs::write(
            src_dir.join("core.rs"),
            r#"
pub mod engine;
pub mod config;

pub struct Core {
    pub config: config::Config,
}
"#,
        )
        .unwrap();

        // Engine module
        fs::write(
            core_dir.join("engine.rs"),
            r#"
use super::config::Config;

pub fn run() {
    let config = Config::default();
    println!("Running with config: {:?}", config);
}
"#,
        )
        .unwrap();

        // Config module
        fs::write(
            core_dir.join("config.rs"),
            r#"
#[derive(Debug, Default)]
pub struct Config {
    pub debug: bool,
    pub threads: usize,
}
"#,
        )
        .unwrap();

        // Utils module
        fs::write(
            src_dir.join("utils.rs"),
            r#"
pub fn helper() -> String {
    "helper function".to_string()
}
"#,
        )
        .unwrap();

        let bundler = Bundler::new();
        let result = bundler.bundle(&project_path).unwrap();

        // Verify all modules are included
        assert!(result.contains("fn main"));
        assert!(result.contains("pub fn run"));
        assert!(result.contains("struct Config"));
        assert!(result.contains("pub fn helper"));

        // Verify it's valid Rust
        syn::parse_file(&result).unwrap();
    }
}
