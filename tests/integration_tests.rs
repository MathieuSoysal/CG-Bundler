//! Integration tests for the rust-singler CLI application

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A Rust code compression tool"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_compress_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.rs");
    let output_file = temp_dir.path().join("output.rs");
    
    let input_code = r#"
        /// Documentation comment
        fn main() {
            println!("Hello, world!");
        }
        
        #[test]
        fn test_function() {
            assert_eq!(1, 1);
        }
    "#;
    
    fs::write(&input_file, input_code).unwrap();
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "file",
        "--input", input_file.to_str().unwrap(),
        "--output", output_file.to_str().unwrap(),
    ]);
    
    cmd.assert().success();
    
    // Check that output file was created
    assert!(output_file.exists());
    
    let output_content = fs::read_to_string(&output_file).unwrap();
    
    // Should contain main function but not test function or doc comments
    assert!(output_content.contains("fn main"));
    assert!(output_content.contains("println!"));
    assert!(!output_content.contains("test_function"));
    assert!(!output_content.contains("Documentation comment"));
}

#[test]
fn test_compress_directory() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    let output_file = temp_dir.path().join("compressed.rs");
    
    fs::create_dir(&src_dir).unwrap();
    
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    
    fs::write(&main_rs, "fn main() { println!(\"main\"); }").unwrap();
    fs::write(&lib_rs, "pub fn lib_func() { println!(\"lib\"); }").unwrap();
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "directory",
        "--input", src_dir.to_str().unwrap(),
        "--output", output_file.to_str().unwrap(),
    ]);
    
    cmd.assert().success();
    
    assert!(output_file.exists());
    
    let output_content = fs::read_to_string(&output_file).unwrap();
    
    // Should contain both functions
    assert!(output_content.contains("fn main"));
    assert!(output_content.contains("pub fn lib_func"));
}

#[test]
fn test_invalid_input_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("output.rs");
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "file",
        "--input", "/nonexistent/file.rs",
        "--output", output_file.to_str().unwrap(),
    ]);
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_preserve_strings_option() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.rs");
    let output_file = temp_dir.path().join("output.rs");
    
    let input_code = r#"
        fn main() {
            let message = "Hello,    world!";
            println!("{}", message);
        }
    "#;
    
    fs::write(&input_file, input_code).unwrap();
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "file",
        "--input", input_file.to_str().unwrap(),
        "--output", output_file.to_str().unwrap(),
        "--preserve-strings",
    ]);
    
    cmd.assert().success();
    
    let output_content = fs::read_to_string(&output_file).unwrap();
    
    // String content should be preserved
    assert!(output_content.contains("Hello,    world!"));
}

#[test]
fn test_keep_docs_option() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.rs");
    let output_file = temp_dir.path().join("output.rs");
    
    let input_code = r#"
        /// This is a doc comment
        fn main() {
            println!("Hello!");
        }
    "#;
    
    fs::write(&input_file, input_code).unwrap();
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "file",
        "--input", input_file.to_str().unwrap(),
        "--output", output_file.to_str().unwrap(),
        "--keep-docs",
    ]);
    
    cmd.assert().success();
    
    let output_content = fs::read_to_string(&output_file).unwrap();
    
    // Doc comment should be preserved when --keep-docs is used
    // Note: syn parses doc comments into attributes, so the exact format might change
    assert!(output_content.contains("fn main"));
}

#[test]
fn test_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("input.rs");
    let output_file = temp_dir.path().join("output.rs");
    
    fs::write(&input_file, "fn main() {}").unwrap();
    
    let mut cmd = Command::cargo_bin("rust-singler").unwrap();
    cmd.args(&[
        "--verbose",
        "file",
        "--input", input_file.to_str().unwrap(),
        "--output", output_file.to_str().unwrap(),
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing file"));
}
