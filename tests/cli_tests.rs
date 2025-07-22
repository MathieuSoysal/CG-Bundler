use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Tests for CLI functionality to improve main.rs coverage
mod cli_functionality_tests {
    use super::*;

    fn create_test_project(project_path: &Path, name: &str, content: &str) {
        fs::create_dir_all(project_path.join("src")).expect("Failed to create src");

        fs::write(
            project_path.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
                name
            ),
        )
        .expect("Failed to write Cargo.toml");

        fs::write(project_path.join("src/main.rs"), content).expect("Failed to write main.rs");
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("cg-bundler"));
    }

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("A Rust code bundler"))
            .stdout(predicate::str::contains("Usage:"));
    }

    #[test]
    fn test_cli_basic_bundling() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("fn main"))
            .stdout(predicate::str::contains("println!"));
    }

    #[test]
    fn test_cli_output_to_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");
        let output_file = temp_dir.path().join("output.rs");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--output")
            .arg(&output_file)
            .assert()
            .success();

        // Check that output file was created and contains expected content
        assert!(output_file.exists(), "Output file should be created");

        let content = fs::read_to_string(&output_file).expect("Should read output file");
        assert!(
            content.contains("fn main"),
            "Output should contain main function"
        );
        assert!(
            content.contains("println!"),
            "Output should contain println!"
        );
    }

    #[test]
    fn test_cli_with_verbose_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--verbose")
            .assert()
            .success()
            .stderr(predicate::str::contains("Bundling project")); // Verbose output to stderr
    }

    #[test]
    fn test_cli_with_validate_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--validate")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_with_keep_tests_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            r#"
fn main() { 
    println!("Hello, world!"); 
}

#[test]
fn test_function() {
    assert!(true);
}
"#,
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--keep-tests")
            .assert()
            .success()
            .stdout(predicate::str::contains("#[test]"))
            .stdout(predicate::str::contains("test_function"));
    }

    #[test]
    fn test_cli_with_keep_docs_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            r#"
/// This is the main function
fn main() { 
    println!("Hello, world!"); 
}
"#,
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--keep-docs")
            .assert()
            .success()
            .stdout(predicate::str::contains("/// This is the main function"));
    }

    #[test]
    fn test_cli_with_no_expand_modules_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "mod utils;\nfn main() { println!(\"Hello, world!\"); }",
        );

        // Create utils module
        fs::write(
            project_path.join("src/utils.rs"),
            "pub fn helper() { println!(\"Helper function\"); }",
        )
        .expect("Failed to write utils.rs");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--no-expand-modules")
            .assert()
            .success()
            .stdout(predicate::str::contains("mod utils")); // Should keep module declaration
    }

    #[test]
    fn test_cli_with_minify_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { \n    println!(\"Hello, world!\"); \n}",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--minify")
            .assert()
            .success();
        // Note: Minification effects are hard to test without knowing exact implementation
    }

    #[test]
    fn test_cli_with_m2_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { \n    println!(\"Hello, world!\"); \n}",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--m2")
            .assert()
            .success();
    }

    #[test]
    fn test_cli_with_invalid_project_path() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir("/tmp") // Non-Rust project directory
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:")); // Should show error message
    }

    #[test]
    fn test_cli_with_project_path_argument() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg(&project_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("fn main"))
            .stdout(predicate::str::contains("println!"));
    }

    #[test]
    fn test_cli_with_multiple_flags() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");
        let output_file = temp_dir.path().join("output.rs");

        create_test_project(
            &project_path,
            "test_project",
            r#"
/// Main function
fn main() { 
    println!("Hello, world!"); 
}

#[test]
fn test_function() {
    assert!(true);
}
"#,
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--verbose")
            .arg("--keep-tests")
            .arg("--keep-docs")
            .arg("--output")
            .arg(&output_file)
            .assert()
            .success()
            .stderr(predicate::str::contains("Bundle complete")); // Verbose output

        // Check output file contains everything
        let content = fs::read_to_string(&output_file).expect("Should read output file");
        assert!(content.contains("/// Main function"), "Should keep docs");
        assert!(content.contains("#[test]"), "Should keep tests");
        assert!(content.contains("fn main"), "Should have main function");
    }

    #[test]
    fn test_cli_error_handling_with_malformed_rust() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { invalid rust syntax !!!",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:")); // Should show parse error
    }

    #[test]
    fn test_cli_with_complex_project_structure() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("complex_project");

        fs::create_dir_all(project_path.join("src/modules")).expect("Failed to create modules");

        fs::write(
            project_path.join("Cargo.toml"),
            r#"
[package]
name = "complex_project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::write(
            project_path.join("src/main.rs"),
            r#"
mod modules;
use modules::utils::helper;

fn main() {
    helper();
    println!("Hello from complex project!");
}
"#,
        )
        .expect("Failed to write main.rs");

        fs::write(project_path.join("src/modules/mod.rs"), "pub mod utils;")
            .expect("Failed to write modules/mod.rs");

        fs::write(
            project_path.join("src/modules/utils.rs"),
            "pub fn helper() { println!(\"Helper function called\"); }",
        )
        .expect("Failed to write modules/utils.rs");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .assert()
            .success()
            .stdout(predicate::str::contains("fn main"))
            .stdout(predicate::str::contains("helper")); // Should expand modules by default
    }
}
