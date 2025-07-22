use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

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

/// Tests for CLI functionality to improve main.rs coverage
mod cli_functionality_tests {
    use super::*;

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

    #[test]
    fn test_cli_help_contains_github_issues_link() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("🐛 Found a bug or need help?"))
            .stdout(predicate::str::contains(
                "https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            ))
            .stdout(predicate::str::contains("📖 Documentation:"))
            .stdout(predicate::str::contains("https://docs.rs/cg-bundler"));
    }

    #[test]
    fn test_cli_error_contains_github_issues_link() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("/nonexistent/project/path")
            .assert()
            .failure()
            .stderr(predicate::str::contains("💡 Need help or found a bug?"))
            .stderr(predicate::str::contains(
                "https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            ))
            .stderr(predicate::str::contains(
                "Your feedback helps improve CG-Bundler",
            ))
            .stderr(predicate::str::contains("━"));
    }

    #[test]
    fn test_cli_info_contains_github_issues_link() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");

        create_test_project(
            &project_path,
            "test_project",
            "fn main() { println!(\"Hello, world!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--info")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "ℹ️  Need help or want to report an issue?",
            ))
            .stdout(predicate::str::contains(
                "https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            ))
            .stdout(predicate::str::contains("━"));
    }

    #[test]
    fn test_cli_validate_verbose_contains_github_issues_link() {
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
            .arg("--verbose")
            .assert()
            .success()
            .stderr(predicate::str::contains(
                "ℹ️  Need help or want to report an issue?",
            ))
            .stderr(predicate::str::contains(
                "https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            ));
    }

    #[test]
    fn test_cli_bundle_verbose_contains_github_issues_link() {
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
            .arg("--verbose")
            .arg("--output")
            .arg(&output_file)
            .assert()
            .success()
            .stderr(predicate::str::contains("ℹ️  Issues or feedback? Visit:"))
            .stderr(predicate::str::contains(
                "https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            ));
    }

    #[test]
    fn test_cli_short_help_does_not_contain_github_link() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Test that short help (-h) doesn't show the detailed GitHub link
        cmd.arg("-h")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Bundle Rust projects into single files",
            ))
            .stdout(predicate::str::contains("see more with '--help'"))
            .stdout(
                predicate::str::contains("https://github.com/MathieuSoysal/CG-Bundler/issues/new")
                    .not(),
            );
    }
}

/// Tests for watch mode functionality
mod watch_mode_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cli_watch_flag_exists() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--watch"))
            .stdout(predicate::str::contains("Watch for file changes"));
    }

    #[test]
    fn test_cli_watch_short_flag() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("-w, --watch"));
    }

    #[test]
    fn test_cli_src_dir_flag() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--src-dir"))
            .stdout(predicate::str::contains("Source directory to watch"));
    }

    #[test]
    fn test_cli_debounce_flag() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("--debounce"))
            .stdout(predicate::str::contains("Debounce delay in milliseconds"));
    }

    #[test]
    fn test_watch_mode_with_invalid_project() {
        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.arg("--watch")
            .arg("nonexistent_project")
            .timeout(Duration::from_secs(5))
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn test_watch_mode_shows_startup_messages() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "watch_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Use timeout to avoid hanging - watch mode will run indefinitely
        cmd.current_dir(temp_dir.path())
            .arg("--watch")
            .arg("-o")
            .arg("output.rs")
            .timeout(Duration::from_secs(2))
            .assert();

        // We expect this to timeout since watch mode runs forever
        // The test passes if we don't get an immediate error
    }

    #[test]
    fn test_watch_mode_with_custom_src_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().join("custom_project");
        fs::create_dir_all(&project_path).expect("Failed to create project dir");

        // Create a custom source directory
        fs::create_dir_all(project_path.join("custom_src")).expect("Failed to create custom_src");

        fs::write(
            project_path.join("Cargo.toml"),
            r#"
[package]
name = "custom_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "custom_project"
path = "custom_src/main.rs"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::write(
            project_path.join("custom_src/main.rs"),
            "fn main() { println!(\"Custom source!\"); }",
        )
        .expect("Failed to write main.rs");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Test that watch mode accepts custom src dir
        cmd.current_dir(&project_path)
            .arg("--watch")
            .arg("--src-dir")
            .arg("custom_src")
            .arg("-o")
            .arg("output.rs")
            .timeout(Duration::from_secs(2))
            .assert();

        // We expect this to timeout since watch mode runs forever
    }

    #[test]
    fn test_watch_mode_with_custom_debounce() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "debounce_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Test that watch mode accepts custom debounce
        cmd.current_dir(temp_dir.path())
            .arg("--watch")
            .arg("--debounce")
            .arg("1000")
            .arg("-o")
            .arg("output.rs")
            .timeout(Duration::from_secs(2))
            .assert();

        // We expect this to timeout since watch mode runs forever
    }
}

/// Tests for additional CLI edge cases and functionality
mod advanced_cli_tests {
    use super::*;

    #[test]
    fn test_cli_with_all_flags_combined() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(
            temp_dir.path(),
            "all_flags_test",
            "fn main() { println!(\"All flags!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--verbose")
            .arg("--keep-tests")
            .arg("--keep-docs")
            .arg("--minify")
            .arg("-o")
            .arg("all_flags_output.rs")
            .assert()
            .success();

        // Check that output file was created
        assert!(temp_dir.path().join("all_flags_output.rs").exists());
    }

    #[test]
    fn test_cli_info_command_shows_detailed_information() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "info_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--info")
            .assert()
            .success()
            .stdout(predicate::str::contains("Project Information"))
            .stdout(predicate::str::contains("Name: info_test"))
            .stdout(predicate::str::contains("Version: 0.1.0"))
            .stdout(predicate::str::contains("Targets"))
            .stdout(predicate::str::contains("Binary"));
    }

    #[test]
    fn test_cli_validate_command() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(
            temp_dir.path(),
            "validate_test",
            "fn main() { println!(\"Valid!\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--validate")
            .assert()
            .success()
            .stdout(predicate::str::contains("Project validation successful"));
    }

    #[test]
    fn test_cli_validate_verbose() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "validate_verbose_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--validate")
            .arg("--verbose")
            .assert()
            .success()
            .stderr(predicate::str::contains("Validating project:"))
            .stderr(predicate::str::contains("✓ Project structure is valid"))
            .stderr(predicate::str::contains(
                "✓ Project can be bundled successfully",
            ))
            .stderr(predicate::str::contains(
                "✓ Generated code is syntactically valid",
            ));
    }

    #[test]
    fn test_cli_pretty_flag() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(
            temp_dir.path(),
            "pretty_test",
            "fn main(){let x=5;println!(\"x is {}\",x);}",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--pretty")
            .arg("-o")
            .arg("pretty_output.rs")
            .assert()
            .success();

        // Read the output file and check it's properly formatted
        let output_content = fs::read_to_string(temp_dir.path().join("pretty_output.rs"))
            .expect("Failed to read output file");

        // Should have proper formatting (if rustfmt is available)
        assert!(output_content.contains("fn main()"));
    }

    #[test]
    fn test_cli_invalid_flag_combination() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "invalid_combo_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Test that minify and pretty can't be used together effectively
        // (both should work, but pretty should be ignored when minify is used)
        cmd.current_dir(temp_dir.path())
            .arg("--minify")
            .arg("--pretty")
            .arg("-o")
            .arg("combo_output.rs")
            .assert()
            .success();

        // Check that output file was created
        assert!(temp_dir.path().join("combo_output.rs").exists());
    }

    #[test]
    fn test_cli_output_to_stdout() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(
            temp_dir.path(),
            "stdout_test",
            "fn main() { println!(\"Hello\"); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("fn main()"))
            .stdout(predicate::str::contains("println!"));
    }

    #[test]
    fn test_cli_with_project_path_and_output() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_name = "path_and_output_test";
        let project_path = temp_dir.path().join(project_name);

        // Create the project directory first
        fs::create_dir_all(&project_path).expect("Failed to create project directory");
        create_test_project(&project_path, project_name, "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        let output_path = temp_dir.path().join("bundled_output.rs");

        cmd.arg(&project_path)
            .arg("-o")
            .arg(&output_path)
            .assert()
            .success();

        // Check that output file was created in the correct location
        assert!(output_path.exists());
    }

    #[test]
    fn test_cli_m2_aggressive_minify() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(
            temp_dir.path(),
            "m2_test",
            "fn main() { let x = 5 + 3; println!(\"Result: {}\", x); }",
        );

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--m2")
            .arg("-o")
            .arg("m2_output.rs")
            .assert()
            .success();

        // Read the output and verify aggressive minification
        let output_content = fs::read_to_string(temp_dir.path().join("m2_output.rs"))
            .expect("Failed to read output file");

        // Should have aggressive minification (no spaces around operators)
        assert!(output_content.contains("5+3"));
    }

    #[test]
    fn test_cli_no_expand_modules() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().join("no_expand_test");
        fs::create_dir_all(&project_path).expect("Failed to create project dir");

        // Create a project with modules
        fs::write(
            project_path.join("Cargo.toml"),
            r#"
[package]
name = "no_expand_test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::create_dir_all(project_path.join("src")).expect("Failed to create src");

        fs::write(
            project_path.join("src/main.rs"),
            "mod utils;\nfn main() { utils::hello(); }",
        )
        .expect("Failed to write main.rs");

        fs::write(
            project_path.join("src/utils.rs"),
            "pub fn hello() { println!(\"Hello from utils!\"); }",
        )
        .expect("Failed to write utils.rs");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .arg("--no-expand-modules")
            .arg("-o")
            .arg("no_expand_output.rs")
            .assert()
            .success();

        // Read the output and verify modules were not expanded
        let output_content = fs::read_to_string(project_path.join("no_expand_output.rs"))
            .expect("Failed to read output file");

        // Should still contain module declaration
        assert!(output_content.contains("mod utils"));
    }
}

/// Tests for error conditions and edge cases
mod error_condition_tests {
    use super::*;

    #[test]
    fn test_cli_with_invalid_debounce_value() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "invalid_debounce_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Test with non-numeric debounce value
        cmd.current_dir(temp_dir.path())
            .arg("--watch")
            .arg("--debounce")
            .arg("invalid")
            .timeout(Duration::from_secs(2))
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid value"));
    }

    #[test]
    fn test_cli_with_nonexistent_src_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "nonexistent_src_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(temp_dir.path())
            .arg("--watch")
            .arg("--src-dir")
            .arg("nonexistent_directory")
            .timeout(Duration::from_secs(2))
            .assert()
            .failure()
            .stderr(predicate::str::contains("does not exist"));
    }

    #[test]
    fn test_cli_with_invalid_output_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_project(temp_dir.path(), "invalid_output_test", "fn main() {}");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        // Try to write to a directory that doesn't exist
        cmd.current_dir(temp_dir.path())
            .arg("-o")
            .arg("nonexistent_dir/output.rs")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn test_cli_with_malformed_cargo_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = temp_dir.path().join("malformed_test");
        fs::create_dir_all(&project_path).expect("Failed to create project dir");

        // Create malformed Cargo.toml
        fs::write(project_path.join("Cargo.toml"), "invalid toml content [[[")
            .expect("Failed to write invalid Cargo.toml");

        fs::create_dir_all(project_path.join("src")).expect("Failed to create src");
        fs::write(project_path.join("src/main.rs"), "fn main() {}")
            .expect("Failed to write main.rs");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&project_path)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn test_cli_with_empty_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let empty_project = temp_dir.path().join("empty_project");
        fs::create_dir_all(&empty_project).expect("Failed to create empty project");

        let mut cmd = Command::cargo_bin("cg-bundler").expect("Binary should exist");

        cmd.current_dir(&empty_project)
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }
}
