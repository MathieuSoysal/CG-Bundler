use cg_bundler::error::BundlerError;
use cg_bundler::file_manager::FileManager;
use cg_bundler::transformer::{CodeTransformer, TransformConfig};
use cg_bundler::Bundler;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Unit tests for the TransformConfig struct
mod transform_config_tests {
    use super::*;

    #[test]
    fn test_default_transform_config() {
        let config = TransformConfig::default();

        assert!(config.remove_tests);
        assert!(config.remove_docs);
        assert!(config.expand_modules);
        assert!(!config.minify);
        assert!(!config.aggressive_minify);
    }

    #[test]
    fn test_custom_transform_config() {
        let config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: false,
            minify: true,
            aggressive_minify: true,
        };

        assert!(!config.remove_tests);
        assert!(!config.remove_docs);
        assert!(!config.expand_modules);
        assert!(config.minify);
        assert!(config.aggressive_minify);
    }

    #[test]
    fn test_config_clone() {
        let config1 = TransformConfig {
            remove_tests: true,
            remove_docs: false,
            expand_modules: true,
            minify: false,
            aggressive_minify: true,
        };

        let config2 = config1.clone();

        assert_eq!(config1.remove_tests, config2.remove_tests);
        assert_eq!(config1.remove_docs, config2.remove_docs);
        assert_eq!(config1.expand_modules, config2.expand_modules);
        assert_eq!(config1.minify, config2.minify);
        assert_eq!(config1.aggressive_minify, config2.aggressive_minify);
    }

    #[test]
    fn test_config_debug() {
        let config = TransformConfig::default();
        let debug_string = format!("{:?}", config);

        assert!(debug_string.contains("TransformConfig"));
        assert!(debug_string.contains("remove_tests"));
        assert!(debug_string.contains("remove_docs"));
        assert!(debug_string.contains("expand_modules"));
        assert!(debug_string.contains("minify"));
        assert!(debug_string.contains("aggressive_minify"));
    }
}

/// Unit tests for the BundlerError enum
mod bundler_error_tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_with_path() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let path = std::path::PathBuf::from("/test/path.rs");

        let bundler_error = BundlerError::Io {
            source: io_err,
            path: Some(path.clone()),
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("/test/path.rs"));
        assert!(error_string.contains("File not found"));
    }

    #[test]
    fn test_io_error_without_path() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");

        let bundler_error = BundlerError::Io {
            source: io_err,
            path: None,
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("Permission denied"));
        assert!(!error_string.contains("/"));
    }

    #[test]
    fn test_parsing_error_with_path() {
        let path = std::path::PathBuf::from("/src/main.rs");

        let bundler_error = BundlerError::Parsing {
            message: "Unexpected token".to_string(),
            file_path: Some(path.clone()),
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Parsing error"));
        assert!(error_string.contains("/src/main.rs"));
        assert!(error_string.contains("Unexpected token"));
    }

    #[test]
    fn test_parsing_error_without_path() {
        let bundler_error = BundlerError::Parsing {
            message: "Invalid syntax".to_string(),
            file_path: None,
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Parsing error"));
        assert!(error_string.contains("Invalid syntax"));
        assert!(!error_string.contains("/"));
    }

    #[test]
    fn test_cargo_metadata_error() {
        let bundler_error = BundlerError::CargoMetadata {
            message: "Failed to parse Cargo.toml".to_string(),
            source: None,
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Cargo metadata error"));
        assert!(error_string.contains("Failed to parse Cargo.toml"));
    }

    #[test]
    fn test_project_structure_error() {
        let bundler_error = BundlerError::ProjectStructure {
            message: "Invalid project layout".to_string(),
        };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Project structure error"));
        assert!(error_string.contains("Invalid project layout"));
    }

    #[test]
    fn test_multiple_binary_targets_error() {
        let bundler_error = BundlerError::MultipleBinaryTargets { target_count: 3 };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Multiple binary targets"));
        assert!(error_string.contains("3"));
    }

    #[test]
    fn test_no_binary_target_error() {
        let bundler_error = BundlerError::NoBinaryTarget;

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("No binary target"));
    }

    #[test]
    fn test_multiple_library_targets_error() {
        let bundler_error = BundlerError::MultipleLibraryTargets { target_count: 2 };

        let error_string = format!("{}", bundler_error);
        assert!(error_string.contains("Multiple library targets"));
        assert!(error_string.contains("2"));
    }

    #[test]
    fn test_error_debug_format() {
        let bundler_error = BundlerError::ProjectStructure {
            message: "Test error".to_string(),
        };

        let debug_string = format!("{:?}", bundler_error);
        assert!(debug_string.contains("ProjectStructure"));
        assert!(debug_string.contains("Test error"));
    }

    #[test]
    fn test_error_source_trait() {
        use std::error::Error;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let bundler_error = BundlerError::Io {
            source: io_err,
            path: None,
        };

        // Test that BundlerError implements the Error trait
        let _: &dyn Error = &bundler_error;
    }
}

/// Unit tests for the FileManager
mod file_manager_tests {
    use super::*;

    #[test]
    fn test_read_existing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test.txt");

        let content = "Hello, world!\nThis is a test file.";
        fs::write(&file_path, content).expect("Failed to write test file");

        let read_content = FileManager::read_file(&file_path).expect("Failed to read file");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = FileManager::read_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_empty_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("empty.txt");

        fs::write(&file_path, "").expect("Failed to write empty file");

        let read_content = FileManager::read_file(&file_path).expect("Failed to read empty file");
        assert_eq!(read_content, "");
    }

    #[test]
    fn test_read_file_with_unicode() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("unicode.txt");

        let content = "Hello 世界! 🦀 Rust is awesome! éñüñ";
        fs::write(&file_path, content).expect("Failed to write unicode file");

        let read_content = FileManager::read_file(&file_path).expect("Failed to read unicode file");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_large_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("large.txt");

        // Create a large file (100KB)
        let line = "This is a line of text that will be repeated many times.\n";
        let mut large_content = String::new();
        for _ in 0..2000 {
            large_content.push_str(line);
        }

        fs::write(&file_path, &large_content).expect("Failed to write large file");

        let read_content = FileManager::read_file(&file_path).expect("Failed to read large file");
        assert_eq!(read_content, large_content);
        assert!(read_content.len() > 100_000);
    }

    #[test]
    fn test_read_file_with_special_characters() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("special.txt");

        let content = "Special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?\n\t\r\n";
        fs::write(&file_path, content).expect("Failed to write special chars file");

        let read_content =
            FileManager::read_file(&file_path).expect("Failed to read special chars file");
        assert_eq!(read_content, content);
    }
}

/// Unit tests for CodeTransformer
mod code_transformer_tests {
    use super::*;

    #[test]
    fn test_code_transformer_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();
        let crate_name = "test_crate";
        let config = TransformConfig::default();

        let _transformer = CodeTransformer::new(base_path, crate_name, config.clone());

        // We can't directly access private fields, but we can test that creation succeeds
        // and the transformer is ready to use
        assert!(true, "CodeTransformer created successfully");
    }

    #[test]
    fn test_transform_simple_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();
        let crate_name = "test_crate";
        let config = TransformConfig::default();

        let mut transformer = CodeTransformer::new(base_path, crate_name, config);

        // Create a simple Rust file AST
        let code = r#"
fn main() {
    println!("Hello, world!");
}
"#;

        let mut file = syn::parse_file(code).expect("Failed to parse test code");
        let result = transformer.transform_file(&mut file);

        assert!(result.is_ok(), "Transform should succeed for simple file");
    }

    #[test]
    fn test_transform_file_with_docs() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();
        let crate_name = "test_crate";

        // Config that removes docs
        let config = TransformConfig {
            remove_docs: true,
            ..TransformConfig::default()
        };

        let mut transformer = CodeTransformer::new(base_path, crate_name, config);

        let code = r#"
/// This is a documented function
fn documented_function() {
    println!("Hello!");
}
"#;

        let mut file = syn::parse_file(code).expect("Failed to parse test code");
        let result = transformer.transform_file(&mut file);

        assert!(result.is_ok(), "Transform should succeed");

        // Convert back to string to check if docs were removed
        let transformed = prettyplease::unparse(&file);
        assert!(
            !transformed.contains("This is a documented function"),
            "Documentation should be removed"
        );
    }

    #[test]
    fn test_transform_file_with_tests() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();
        let crate_name = "test_crate";

        // Config that removes tests
        let config = TransformConfig {
            remove_tests: true,
            ..TransformConfig::default()
        };

        let mut transformer = CodeTransformer::new(base_path, crate_name, config);

        let code = r#"
fn regular_function() {
    println!("Regular function");
}

#[test]
fn test_function() {
    assert!(true);
}

#[cfg(test)]
mod tests {
    #[test]
    fn another_test() {
        assert_eq!(2 + 2, 4);
    }
}
"#;

        let mut file = syn::parse_file(code).expect("Failed to parse test code");
        let result = transformer.transform_file(&mut file);

        assert!(result.is_ok(), "Transform should succeed");

        // Convert back to string to check if tests were removed
        let transformed = prettyplease::unparse(&file);
        assert!(
            !transformed.contains("#[test]"),
            "Test attributes should be removed"
        );
        assert!(
            !transformed.contains("test_function"),
            "Test functions should be removed"
        );
        assert!(
            transformed.contains("regular_function"),
            "Regular functions should remain"
        );
    }

    #[test]
    fn test_transform_invalid_syntax() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();
        let crate_name = "test_crate";
        let config = TransformConfig::default();

        let _transformer = CodeTransformer::new(base_path, crate_name, config);

        // This should never happen in practice since syn would fail to parse first,
        // but we test the transformer with a valid but incomplete AST
        let code = "fn incomplete_function() {";

        // This should fail at the syn::parse_file level, not in the transformer
        let parse_result = syn::parse_file(code);
        assert!(parse_result.is_err(), "Should fail to parse invalid syntax");
    }
}

/// Edge case tests for individual components
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_transform_config() {
        // Test config with all features disabled
        let config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: false,
            minify: false,
            aggressive_minify: false,
        };

        assert!(!config.remove_tests);
        assert!(!config.remove_docs);
        assert!(!config.expand_modules);
        assert!(!config.minify);
        assert!(!config.aggressive_minify);
    }

    #[test]
    fn test_maximal_transform_config() {
        // Test config with all features enabled
        let config = TransformConfig {
            remove_tests: true,
            remove_docs: true,
            expand_modules: true,
            minify: true,
            aggressive_minify: true,
        };

        assert!(config.remove_tests);
        assert!(config.remove_docs);
        assert!(config.expand_modules);
        assert!(config.minify);
        assert!(config.aggressive_minify);
    }

    #[test]
    fn test_error_with_very_long_message() {
        let long_message = "a".repeat(10000);

        let error = BundlerError::ProjectStructure {
            message: long_message.clone(),
        };

        let error_string = format!("{}", error);
        assert!(error_string.contains(&long_message));
        assert!(error_string.len() > 10000);
    }

    #[test]
    fn test_error_with_special_characters() {
        let special_message = "Error with special chars: 🦀 ñ é ü ❤️ \n\t\r";

        let error = BundlerError::ProjectStructure {
            message: special_message.to_string(),
        };

        let error_string = format!("{}", error);
        assert!(error_string.contains("🦀"));
        assert!(error_string.contains("ñ"));
    }

    #[test]
    fn test_file_manager_with_empty_path() {
        let empty_path = Path::new("");
        let result = FileManager::read_file(empty_path);

        assert!(result.is_err(), "Should fail to read empty path");
    }

    #[test]
    fn test_file_manager_with_directory_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir_path = temp_dir.path();

        let result = FileManager::read_file(dir_path);
        assert!(result.is_err(), "Should fail to read directory as file");
    }
}

// ==============================================
// ADDITIONAL UNIT TESTS
// ==============================================

/// Tests for error propagation and handling
mod error_propagation_tests {
    use super::*;
    use std::error::Error;
    use std::path::PathBuf;

    #[test]
    fn test_error_chain_preservation() {
        // Test that errors maintain their source chain
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let bundler_error = BundlerError::Io {
            source: io_error,
            path: Some(PathBuf::from("/restricted/file.rs")),
        };

        // Check that the source is preserved
        let source = bundler_error.source();
        assert!(source.is_some(), "Error should have a source");

        if let Some(source) = source {
            assert_eq!(source.to_string(), "Access denied");
        }
    }

    #[test]
    fn test_error_context_information() {
        let parse_error = BundlerError::Parsing {
            message: "Unexpected token".to_string(),
            file_path: Some(PathBuf::from("/project/src/main.rs")),
        };

        let error_string = format!("{}", parse_error);
        assert!(
            error_string.contains("Unexpected token"),
            "Should contain error message"
        );
        assert!(
            error_string.contains("main.rs"),
            "Should contain file information"
        );
    }

    #[test]
    fn test_error_serialization_compatibility() {
        // Test that error types can be properly formatted and debugged
        let errors = vec![
            BundlerError::ProjectStructure {
                message: "Test message".to_string(),
            },
            BundlerError::CargoMetadata {
                message: "Cargo error".to_string(),
                source: None,
            },
        ];

        for error in errors {
            let debug_str = format!("{:?}", error);
            let display_str = format!("{}", error);

            assert!(!debug_str.is_empty(), "Debug string should not be empty");
            assert!(
                !display_str.is_empty(),
                "Display string should not be empty"
            );
        }
    }
}

/// Tests for transformer internal logic
mod transformer_internals_tests {
    use super::*;

    #[test]
    fn test_transform_config_validation() {
        let configs = vec![
            TransformConfig {
                remove_tests: true,
                remove_docs: true,
                expand_modules: true,
                minify: false,
                aggressive_minify: false,
            },
            TransformConfig {
                remove_tests: false,
                remove_docs: false,
                expand_modules: false,
                minify: true,
                aggressive_minify: true,
            },
        ];

        for (i, config) in configs.iter().enumerate() {
            // Test that aggressive_minify implies minify doesn't break anything
            if config.aggressive_minify && !config.minify {
                // This is a valid configuration where aggressive_minify can work
                // independently or imply minify at a higher level
            }

            let cloned = config.clone();
            assert_eq!(
                config.remove_tests, cloned.remove_tests,
                "Config {} should clone correctly",
                i
            );
        }
    }

    #[test]
    fn test_transformer_creation_with_different_configs() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        let configs = vec![
            TransformConfig::default(),
            TransformConfig {
                remove_tests: false,
                remove_docs: false,
                expand_modules: false,
                minify: true,
                aggressive_minify: false,
            },
        ];

        for (i, config) in configs.iter().enumerate() {
            let _transformer = CodeTransformer::new(temp_dir.path(), "test_crate", config.clone());
            // Just test that creation succeeds
            assert!(true, "Transformer {} should be created successfully", i);
        }
    }
}

/// Tests for file manager edge cases and error handling
mod file_manager_robustness_tests {
    use super::*;

    #[test]
    fn test_file_reading_with_different_line_endings() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test Unix line endings
        let unix_content = "line1\nline2\nline3\n";
        let unix_file = temp_dir.path().join("unix.txt");
        fs::write(&unix_file, unix_content).expect("Failed to write Unix file");

        let read_content = FileManager::read_file(&unix_file).expect("Should read Unix file");
        assert_eq!(read_content, unix_content);

        // Test Windows line endings
        let windows_content = "line1\r\nline2\r\nline3\r\n";
        let windows_file = temp_dir.path().join("windows.txt");
        fs::write(&windows_file, windows_content).expect("Failed to write Windows file");

        let read_content = FileManager::read_file(&windows_file).expect("Should read Windows file");
        assert_eq!(read_content, windows_content);

        // Test mixed line endings
        let mixed_content = "line1\nline2\r\nline3\rline4\n";
        let mixed_file = temp_dir.path().join("mixed.txt");
        fs::write(&mixed_file, mixed_content).expect("Failed to write mixed file");

        let read_content = FileManager::read_file(&mixed_file).expect("Should read mixed file");
        assert_eq!(read_content, mixed_content);
    }

    #[test]
    fn test_module_finding_with_nested_paths() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create nested module structure
        fs::create_dir_all(base_path.join("src/modules/deep/inner_mod"))
            .expect("Failed to create nested dirs");

        // Test finding deeply nested .rs file
        let nested_content = "pub fn deep_function() {}";
        fs::write(
            base_path.join("src/modules/deep/nested_file.rs"),
            nested_content,
        )
        .expect("Failed to write nested.rs");

        let deep_path = base_path.join("src/modules/deep");
        let result = FileManager::find_module_file(&deep_path, "nested_file");
        assert!(result.is_ok(), "Should find nested .rs file");

        if let Ok((found_path, content)) = result {
            assert_eq!(content, nested_content);
            let path_str = found_path.to_string_lossy();
            assert!(
                path_str.contains("src")
                    && path_str.contains("modules")
                    && path_str.contains("deep"),
                "Path should contain expected components: {}",
                path_str
            );
        }

        // Test finding nested mod.rs with different name
        let mod_content = "pub mod inner;";
        fs::write(
            base_path.join("src/modules/deep/inner_mod/mod.rs"),
            mod_content,
        )
        .expect("Failed to write mod.rs");

        let result = FileManager::find_module_file(&deep_path, "inner_mod");
        assert!(result.is_ok(), "Should find nested mod.rs file");

        if let Ok((found_path, content)) = result {
            assert_eq!(content, mod_content);
            let path_str = found_path.to_string_lossy();
            assert!(
                path_str.contains("src")
                    && path_str.contains("modules")
                    && path_str.contains("deep")
                    && path_str.contains("inner_mod"),
                "Path should contain expected components: {}",
                path_str
            );
        }
    }
}

/// Tests for bundler integration with different project configurations
mod bundler_integration_basic_tests {
    use super::*;

    #[test]
    fn test_bundler_with_minimal_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path();

        // Create minimal Cargo.toml
        let cargo_toml = r#"
[package]
name = "minimal"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        fs::create_dir(project_path.join("src")).expect("Failed to create src");

        // Create minimal main.rs
        let main_rs = "fn main() {}";
        fs::write(project_path.join("src/main.rs"), main_rs).expect("Failed to write main.rs");

        let bundler = Bundler::new();
        let result = bundler.bundle(project_path);

        assert!(result.is_ok(), "Should bundle minimal project");

        if let Ok(code) = result {
            assert!(code.contains("fn main"), "Should contain main function");
        }
    }

    #[test]
    fn test_bundler_state_consistency() {
        let bundler = Bundler::new();
        let config1 = bundler.config().clone();

        // Config should remain consistent across multiple accesses
        let config2 = bundler.config().clone();

        assert_eq!(config1.remove_tests, config2.remove_tests);
        assert_eq!(config1.remove_docs, config2.remove_docs);
        assert_eq!(config1.expand_modules, config2.expand_modules);
        assert_eq!(config1.minify, config2.minify);
        assert_eq!(config1.aggressive_minify, config2.aggressive_minify);
    }

    #[test]
    fn test_bundler_error_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test with invalid project
        let bundler = Bundler::new();
        let result = bundler.bundle(temp_dir.path().join("nonexistent"));
        assert!(result.is_err(), "Should fail with non-existent project");

        // Bundler should still be usable after error
        let project_path = temp_dir.path().join("valid_project");
        fs::create_dir_all(project_path.join("src")).expect("Failed to create project");

        fs::write(
            project_path.join("Cargo.toml"),
            r#"
[package]
name = "valid"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::write(project_path.join("src/main.rs"), "fn main() {}")
            .expect("Failed to write main.rs");

        let result = bundler.bundle(&project_path);
        assert!(result.is_ok(), "Should work with valid project after error");
    }
}

/// Performance benchmarking tests
mod performance_tests {
    use super::*;

    #[test]
    fn test_config_creation_performance() {
        let start = std::time::Instant::now();

        for _ in 0..10000 {
            let _config = TransformConfig::default();
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Config creation should be fast");
    }

    #[test]
    fn test_bundler_creation_performance() {
        let start = std::time::Instant::now();

        for _ in 0..1000 {
            let _bundler = Bundler::new();
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "Bundler creation should be fast"
        );
    }

    #[test]
    fn test_error_creation_performance() {
        let start = std::time::Instant::now();

        for i in 0..1000 {
            let _error = BundlerError::ProjectStructure {
                message: format!("Error {}", i),
            };
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "Error creation should be fast");
    }
}

// ==============================================
// ADVANCED ERROR HANDLING AND RECOVERY TESTS
// ==============================================

/// Test error recovery across multiple operations
mod error_recovery_tests {
    use super::*;

    #[test]
    fn test_cascading_error_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let bundler = Bundler::new();

        // Test multiple failed operations followed by successful ones
        let operations = vec![
            temp_dir.path().join("nonexistent1"),
            temp_dir.path().join("nonexistent2"),
            temp_dir.path().join("nonexistent3"),
        ];

        for path in operations {
            let result = bundler.bundle(&path);
            assert!(result.is_err(), "Should fail with non-existent path");
        }

        // Now test with a valid project
        let valid_path = temp_dir.path().join("valid");
        fs::create_dir_all(valid_path.join("src")).expect("Failed to create src");

        fs::write(
            valid_path.join("Cargo.toml"),
            r#"
[package]
name = "recovery_test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::write(valid_path.join("src/main.rs"), "fn main() {}").expect("Failed to write main.rs");

        let result = bundler.bundle(&valid_path);
        assert!(result.is_ok(), "Should succeed after previous errors");
    }

    #[test]
    fn test_error_context_preservation() {
        // Test that error contexts are properly preserved through the stack
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let invalid_project = temp_dir.path().join("invalid");

        fs::create_dir_all(&invalid_project).expect("Failed to create directory");
        fs::write(invalid_project.join("Cargo.toml"), "invalid toml [[[")
            .expect("Failed to write invalid toml");

        let bundler = Bundler::new();
        let result = bundler.bundle(&invalid_project);

        assert!(result.is_err(), "Should fail with invalid Cargo.toml");

        // Check that error contains contextual information
        let error_msg = format!("{}", result.unwrap_err());
        // The exact error message depends on the implementation, but it should contain context
        assert!(!error_msg.is_empty(), "Error message should not be empty");
    }
}

/// Test file system edge cases and permissions
mod filesystem_edge_cases {
    use super::*;

    #[test]
    fn test_file_with_extreme_sizes() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test with empty file
        let empty_file = temp_dir.path().join("empty.rs");
        fs::write(&empty_file, "").expect("Failed to write empty file");

        let result = FileManager::read_file(&empty_file);
        assert!(result.is_ok(), "Should read empty file successfully");
        assert_eq!(result.unwrap(), "", "Empty file should return empty string");

        // Test with file containing only whitespace
        let whitespace_file = temp_dir.path().join("whitespace.rs");
        fs::write(&whitespace_file, "   \n\t\r\n  ").expect("Failed to write whitespace file");

        let result = FileManager::read_file(&whitespace_file);
        assert!(result.is_ok(), "Should read whitespace file successfully");
        assert_eq!(
            result.unwrap(),
            "   \n\t\r\n  ",
            "Should preserve whitespace exactly"
        );
    }

    #[test]
    fn test_file_with_unusual_names() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test files with special characters in names (where supported by filesystem)
        let special_names = vec![
            "file_with_spaces.rs",
            "file-with-dashes.rs",
            "file.with.dots.rs",
            "file_with_numbers123.rs",
        ];

        for name in special_names {
            let file_path = temp_dir.path().join(name);
            let content = format!("// File: {}\nfn test() {{}}", name);
            fs::write(&file_path, &content).expect("Failed to write special name file");

            let result = FileManager::read_file(&file_path);
            assert!(
                result.is_ok(),
                "Should read file with special name: {}",
                name
            );
            assert_eq!(
                result.unwrap(),
                content,
                "Content should match for file: {}",
                name
            );
        }
    }

    #[test]
    fn test_deeply_nested_module_resolution() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let base_path = temp_dir.path();

        // Create extremely deep nesting
        let mut current_path = base_path.join("src");
        let depth = 10;

        for i in 0..depth {
            current_path = current_path.join(format!("level_{}", i));
            fs::create_dir_all(&current_path).expect("Failed to create deep directory");

            // Create both mod.rs and a sibling module
            let mod_content = format!("pub mod level_{};", i + 1);
            fs::write(current_path.join("mod.rs"), &mod_content).expect("Failed to write mod.rs");

            if i < depth - 1 {
                let next_module = format!("level_{}.rs", i + 1);
                let module_content =
                    format!("// Level {} module\npub fn level_{}() {{}}", i + 1, i + 1);
                fs::write(current_path.join(&next_module), &module_content)
                    .expect("Failed to write module");
            }
        }

        // Test finding modules at various depths
        let mut test_path = base_path.join("src");
        for i in 0..depth - 1 {
            test_path = test_path.join(format!("level_{}", i));
            let result = FileManager::find_module_file(&test_path, &format!("level_{}", i + 1));
            assert!(result.is_ok(), "Should find module at depth {}", i + 1);
        }
    }
}

/// Tests for bundler integration with different project configurations
mod bundler_integration_tests {
    use super::*;

    #[test]
    fn test_bundler_with_minimal_project() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path();

        // Create minimal Cargo.toml
        let cargo_toml = r#"
[package]
name = "minimal"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        fs::create_dir(project_path.join("src")).expect("Failed to create src");

        // Create minimal main.rs
        let main_rs = "fn main() {}";
        fs::write(project_path.join("src/main.rs"), main_rs).expect("Failed to write main.rs");

        let bundler = Bundler::new();
        let result = bundler.bundle(project_path);

        assert!(result.is_ok(), "Should bundle minimal project");

        if let Ok(code) = result {
            assert!(code.contains("fn main"), "Should contain main function");
        }
    }

    #[test]
    fn test_bundler_state_consistency() {
        let bundler = Bundler::new();
        let config1 = bundler.config().clone();

        // Config should remain consistent across multiple accesses
        let config2 = bundler.config().clone();

        assert_eq!(config1.remove_tests, config2.remove_tests);
        assert_eq!(config1.remove_docs, config2.remove_docs);
        assert_eq!(config1.expand_modules, config2.expand_modules);
        assert_eq!(config1.minify, config2.minify);
        assert_eq!(config1.aggressive_minify, config2.aggressive_minify);
    }

    #[test]
    fn test_bundler_error_recovery() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test with invalid project
        let bundler = Bundler::new();
        let result = bundler.bundle(temp_dir.path().join("nonexistent"));
        assert!(result.is_err(), "Should fail with non-existent project");

        // Bundler should still be usable after error
        let project_path = temp_dir.path().join("valid_project");
        fs::create_dir_all(project_path.join("src")).expect("Failed to create project");

        fs::write(
            project_path.join("Cargo.toml"),
            r#"
[package]
name = "valid"
version = "0.1.0"
edition = "2021"
"#,
        )
        .expect("Failed to write Cargo.toml");

        fs::write(project_path.join("src/main.rs"), "fn main() {}")
            .expect("Failed to write main.rs");

        let result = bundler.bundle(&project_path);
        assert!(result.is_ok(), "Should work with valid project after error");
    }
}

// ==============================================
// HELP AND ERROR DISPLAY TESTS
// ==============================================

/// Tests for enhanced help and error display functionality
mod help_and_error_display_tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_github_issues_url_consistency() {
        // Test that the GitHub issues URL is consistent across the codebase
        let expected_url = "https://github.com/MathieuSoysal/CG-Bundler/issues/new";

        // Since we can't directly test the display_bug_report_info function from main.rs,
        // we test it indirectly by ensuring CLI commands contain the URL
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("invalid_project");

        // Create an invalid project to trigger error
        fs::create_dir_all(&project_path).expect("Failed to create directory");

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cg-bundler",
                "--",
                &project_path.to_string_lossy(),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_url),
            "Error output should contain GitHub issues URL"
        );
    }

    #[test]
    fn test_enhanced_error_message_components() {
        // Test that the enhanced error message contains all expected components
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let invalid_path = temp_dir
            .path()
            .join("completely_invalid_project_path_12345");

        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cg-bundler",
                "--",
                &invalid_path.to_string_lossy(),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Test all components of the enhanced error message
        let expected_components = vec![
            "Error:",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            "💡 Need help or found a bug?",
            "Please report issues, request features, or get support at:",
            "🔗 https://github.com/MathieuSoysal/CG-Bundler/issues/new",
            "Your feedback helps improve CG-Bundler for everyone!",
        ];

        for component in expected_components {
            assert!(
                stderr.contains(component),
                "Error message should contain: '{}'",
                component
            );
        }
    }

    #[test]
    fn test_help_message_structure() {
        // Test that the help message has the expected structure
        let output = Command::new("cargo")
            .args(&["run", "--bin", "cg-bundler", "--", "--help"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Test help message structure
        assert!(
            stdout.contains("A Rust code bundler that combines multiple source files"),
            "Should contain main description"
        );

        // Test bug report section
        assert!(
            stdout.contains("🐛 Found a bug or need help?"),
            "Should contain bug report section title"
        );
        assert!(
            stdout
                .contains("Report issues: https://github.com/MathieuSoysal/CG-Bundler/issues/new"),
            "Should contain bug report URL"
        );

        // Test documentation section
        assert!(
            stdout.contains("📖 Documentation:"),
            "Should contain documentation section title"
        );
        assert!(
            stdout.contains("https://docs.rs/cg-bundler"),
            "Should contain documentation URL"
        );
    }

    #[test]
    fn test_visual_separators_consistency() {
        // Test that visual separators are used consistently
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let invalid_path = temp_dir.path().join("invalid");

        let error_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cg-bundler",
                "--",
                &invalid_path.to_string_lossy(),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let error_stderr = String::from_utf8_lossy(&error_output.stderr);

        // Count visual separators in error output
        let separator_count = error_stderr.matches("━").count();
        assert!(
            separator_count >= 120, // Should have two full lines of separators (60 chars each)
            "Error should have visual separators, found {} separator chars",
            separator_count
        );

        // Test info command visual separators
        let info_output = Command::new("cargo")
            .args(&["run", "--bin", "cg-bundler", "--", "--info"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        if info_output.status.success() {
            let info_stdout = String::from_utf8_lossy(&info_output.stdout);
            let info_separator_count = info_stdout.matches("━").count();
            assert!(
                info_separator_count >= 100, // Should have visual separators
                "Info should have visual separators, found {} separator chars",
                info_separator_count
            );
        }
    }

    #[test]
    fn test_emoji_accessibility_features() {
        // Test that emoji indicators are used for accessibility
        let help_output = Command::new("cargo")
            .args(&["run", "--bin", "cg-bundler", "--", "--help"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let help_stdout = String::from_utf8_lossy(&help_output.stdout);

        // Test for emojis that improve accessibility
        assert!(help_stdout.contains("🐛"), "Should contain bug emoji");
        assert!(
            help_stdout.contains("📖"),
            "Should contain documentation emoji"
        );

        // Test error emojis
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let invalid_path = temp_dir.path().join("invalid");

        let error_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "cg-bundler",
                "--",
                &invalid_path.to_string_lossy(),
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to execute command");

        let error_stderr = String::from_utf8_lossy(&error_output.stderr);
        assert!(
            error_stderr.contains("💡"),
            "Should contain lightbulb emoji"
        );
        assert!(error_stderr.contains("🔗"), "Should contain link emoji");
    }
}
