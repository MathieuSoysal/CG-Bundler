#[cfg(test)]
mod debug_tests {
    use crate::bundler::Bundler;
    use crate::transformer::TransformConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn debug_bundling_output() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("debug_test");
        fs::create_dir_all(&project_path).unwrap();

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "debug_test"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "debug_test"
path = "src/main.rs"

[lib]
name = "debug_test"
path = "src/lib.rs"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_path.join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create main.rs with extern crate
        let main_content = r#"
extern crate debug_test;

use debug_test::greet;

fn main() {
    println!("{}", greet("World"));
}
"#;
        fs::write(src_dir.join("main.rs"), main_content).unwrap();

        // Create lib.rs with module and tests
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

        // Create utils.rs module with tests
        let utils_content = r#"
/// Format a greeting message
pub fn format_greeting(name: &str) -> String {
    format!("Hello, {}!", name)
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

        // Test with default config (remove tests and docs)
        let bundler = Bundler::new();
        let result = bundler.bundle(&project_path).unwrap();

        println!("==== DEFAULT CONFIG OUTPUT ====");
        println!("{}", result);
        println!("==== END OUTPUT ====");

        // Test with config that preserves tests and docs
        let config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: true,
        };

        let bundler = Bundler::with_config(config);
        let result2 = bundler.bundle(&project_path).unwrap();

        println!("==== PRESERVE CONFIG OUTPUT ====");
        println!("{}", result2);
        println!("==== END OUTPUT ====");

        // Basic assertions
        assert!(result.contains("fn main"));
        assert!(result.contains("pub fn greet"));
        assert!(result.contains("pub fn format_greeting"));

        // Check test removal in default config
        println!("Contains #[test]: {}", result.contains("#[test]"));
        println!("Contains mod tests: {}", result.contains("mod tests"));

        // Check test preservation in custom config
        println!("Config2 Contains #[test]: {}", result2.contains("#[test]"));
        println!(
            "Config2 Contains mod tests: {}",
            result2.contains("mod tests")
        );
    }
}
