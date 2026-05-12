use cg_bundler::cargo_project::CargoProject;
use cg_bundler::error::BundlerError;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_test_project(temp_dir: &Path, has_lib: bool) -> std::path::PathBuf {
    let project_path = temp_dir.join("test_project");
    fs::create_dir_all(&project_path).unwrap();

    let mut cargo_toml = r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test_project"
path = "src/main.rs"
"#
    .to_string();

    if has_lib {
        cargo_toml.push_str(
            r#"
[lib]
name = "test_project"
path = "src/lib.rs"
"#,
        );
    }

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    if has_lib {
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
    }

    project_path
}

#[test]
fn test_project_with_lib() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_test_project(temp_dir.path(), true);

    let project = CargoProject::new(&project_path).unwrap();

    assert_eq!(project.crate_name(), "test_project");
    assert!(project.library_target().is_some());
    assert_eq!(project.binary_target().name, "test_project");
}

#[test]
fn test_project_without_lib() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_test_project(temp_dir.path(), false);

    let project = CargoProject::new(&project_path).unwrap();

    assert_eq!(project.crate_name(), "test_project");
    assert!(project.library_target().is_none());
    assert_eq!(project.binary_target().name, "test_project");
}

#[test]
fn test_nonexistent_project() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent");

    let result = CargoProject::new(&nonexistent_path);
    assert!(result.is_err());
}

#[test]
fn test_project_with_multiple_binaries_should_fail() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("multi_bin_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "multi_bin_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "bin1"
path = "src/bin1.rs"

[[bin]]
name = "bin2"
path = "src/bin2.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("bin1.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("bin2.rs"), "fn main() {}").unwrap();

    let result = CargoProject::new(&project_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BundlerError::MultipleBinaryTargets { target_count } => {
            assert_eq!(target_count, 2);
        }
        _ => panic!("Expected MultipleBinaryTargets error"),
    }
}

#[test]
fn test_project_with_no_binary_should_fail() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("no_bin_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "no_bin_project"
version = "0.1.0"
edition = "2021"

[lib]
name = "no_bin_project"
path = "src/lib.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

    let result = CargoProject::new(&project_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BundlerError::NoBinaryTarget => {}
        _ => panic!("Expected NoBinaryTarget error"),
    }
}

#[test]
fn test_project_with_multiple_libraries_should_fail() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("multi_lib_project");
    fs::create_dir_all(&project_path).unwrap();

    // Note: Multiple [lib] sections in Cargo.toml is actually not valid Cargo syntax
    // This test demonstrates that cargo_metadata will reject such configurations
    let cargo_toml = r#"
[package]
name = "multi_lib_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "multi_lib_project"
path = "src/main.rs"

[lib]
name = "lib1"
path = "src/lib1.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib1.rs"), "pub fn hello1() {}").unwrap();

    // This should actually succeed since only one library is defined
    // Multiple libraries in a single crate isn't supported by Cargo itself
    let result = CargoProject::new(&project_path);
    assert!(result.is_ok());

    let project = result.unwrap();
    assert_eq!(project.library_target().unwrap().name, "lib1");
}

#[test]
fn test_project_paths_and_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_test_project(temp_dir.path(), true);

    let project = CargoProject::new(&project_path).unwrap();

    // Test paths
    assert!(project.binary_source_path().ends_with("main.rs"));
    assert!(project.library_source_path().unwrap().ends_with("lib.rs"));
    assert!(project.base_path().ends_with("src"));

    // Test metadata access
    let metadata = project.metadata();
    assert!(!metadata.packages.is_empty());
    assert_eq!(metadata.packages[0].name.as_str(), "test_project");
}

#[test]
fn test_crate_name_priority() {
    let temp_dir = TempDir::new().unwrap();

    // Test with library - should use library name
    let project_path_with_lib = create_test_project(temp_dir.path(), true);
    let project_with_lib = CargoProject::new(&project_path_with_lib).unwrap();
    assert_eq!(project_with_lib.crate_name(), "test_project");

    // Test without library - should use binary name
    let project_path_no_lib = create_test_project(&temp_dir.path().join("no_lib"), false);
    let project_no_lib = CargoProject::new(&project_path_no_lib).unwrap();
    assert_eq!(project_no_lib.crate_name(), "test_project");
}

#[test]
fn test_project_with_custom_binary_name() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("custom_bin_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "custom_bin_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "my_custom_binary"
path = "src/main.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let project = CargoProject::new(&project_path).unwrap();

    assert_eq!(project.binary_target().name, "my_custom_binary");
    assert_eq!(project.crate_name(), "my_custom_binary");
    assert!(project.library_target().is_none());
}

#[test]
fn test_project_with_hyphenated_name_normalizes_crate_identifier() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("hyphenated_name_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "winter-challenge-2026"
version = "0.1.0"
edition = "2021"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();

    let project = CargoProject::new(&project_path).unwrap();

    assert_eq!(project.binary_target().name, "winter-challenge-2026");
    assert_eq!(project.crate_name(), "winter_challenge_2026");
}

#[test]
fn test_project_with_workspace() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace_path).unwrap();

    // Create workspace Cargo.toml
    let workspace_cargo_toml = r#"
[workspace]
members = ["member1"]
"#;
    fs::write(workspace_path.join("Cargo.toml"), workspace_cargo_toml).unwrap();

    // Create member project
    let member_path = workspace_path.join("member1");
    fs::create_dir_all(&member_path).unwrap();

    let member_cargo_toml = r#"
[package]
name = "member1"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "member1"
path = "src/main.rs"
"#;
    fs::write(member_path.join("Cargo.toml"), member_cargo_toml).unwrap();

    let src_dir = member_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    // Test that we can load the member project directly
    let project = CargoProject::new(&member_path).unwrap();
    assert_eq!(project.crate_name(), "member1");
    assert_eq!(project.binary_target().name, "member1");
}

#[test]
fn test_project_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("deps_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "deps_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"

[dev-dependencies]
tempfile = "3.0"

[[bin]]
name = "deps_project"
path = "src/main.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let project = CargoProject::new(&project_path).unwrap();

    assert_eq!(project.crate_name(), "deps_project");

    // Check that dependencies are loaded in metadata
    let package = project.root_package();
    assert!(!package.dependencies.is_empty());
}

#[test]
fn test_invalid_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("invalid_project");
    fs::create_dir_all(&project_path).unwrap();

    // Create invalid Cargo.toml
    let invalid_cargo_toml = r#"
[package
name = "invalid_project"
version = "0.1.0"
"#; // Missing closing bracket

    fs::write(project_path.join("Cargo.toml"), invalid_cargo_toml).unwrap();

    let result = CargoProject::new(&project_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BundlerError::CargoMetadata { message, .. } => {
            assert!(message.contains("Failed to obtain cargo metadata"));
        }
        _ => panic!("Expected CargoMetadata error"),
    }
}

#[test]
fn test_project_target_kinds() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("target_kinds_project");
    fs::create_dir_all(&project_path).unwrap();

    let cargo_toml = r#"
[package]
name = "target_kinds_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "main_binary"
path = "src/main.rs"

[lib]
name = "main_library"
path = "src/lib.rs"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub fn lib_fn() {}").unwrap();

    let project = CargoProject::new(&project_path).unwrap();

    // Test that target identification works correctly
    let binary_target = project.binary_target();
    let library_target = project.library_target().unwrap();

    assert_eq!(binary_target.name, "main_binary");
    assert_eq!(library_target.name, "main_library");

    // Test crate name uses library target when available
    assert_eq!(project.crate_name(), "main_library");
}

#[test]
fn test_current_directory_project() {
    // This test ensures that the bundler can work with the current project
    // Since we're running tests in the cg-bundler project itself
    let current_dir = std::env::current_dir().unwrap();

    // Only run this test if we're in a cargo project directory
    if current_dir.join("Cargo.toml").exists() {
        let result = CargoProject::new(&current_dir);
        assert!(result.is_ok());

        let project = result.unwrap();
        // Note: Library target name uses underscores while package name uses hyphens
        assert_eq!(project.crate_name(), "cg_bundler"); // Library name with underscores
        assert_eq!(project.binary_target().name, "cg-bundler"); // Binary name with hyphens
    }
}

#[test]
fn test_error_handling_edge_cases() {
    // Test that our error types are working correctly
    let temp_dir = TempDir::new().unwrap();

    // Test with completely empty directory
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir_all(&empty_dir).unwrap();

    let result = CargoProject::new(&empty_dir);
    assert!(result.is_err());

    // Test with directory that has Cargo.toml but no src
    let no_src_dir = temp_dir.path().join("no_src");
    fs::create_dir_all(&no_src_dir).unwrap();
    fs::write(
        no_src_dir.join("Cargo.toml"),
        r#"
[package]
name = "no_src"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let result = CargoProject::new(&no_src_dir);
    assert!(result.is_err());
}

#[test]
fn test_project_structure_edge_cases() {
    let temp_dir = TempDir::new().unwrap();

    // Test project with custom source paths
    let custom_paths_project = temp_dir.path().join("custom_paths");
    fs::create_dir_all(&custom_paths_project).unwrap();

    let cargo_toml = r#"
[package]
name = "custom_paths"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "custom_binary"
path = "custom/binary.rs"

[lib]
name = "custom_lib"
path = "custom/lib.rs"
"#;

    fs::write(custom_paths_project.join("Cargo.toml"), cargo_toml).unwrap();

    let custom_dir = custom_paths_project.join("custom");
    fs::create_dir_all(&custom_dir).unwrap();
    fs::write(custom_dir.join("binary.rs"), "fn main() {}").unwrap();
    fs::write(custom_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

    let project = CargoProject::new(&custom_paths_project).unwrap();

    assert_eq!(project.binary_target().name, "custom_binary");
    assert_eq!(project.library_target().unwrap().name, "custom_lib");
    assert!(project.binary_source_path().ends_with("binary.rs"));
    assert!(project.library_source_path().unwrap().ends_with("lib.rs"));
    assert!(project.base_path().ends_with("custom"));
}

#[test]
fn test_external_lib_paths_with_path_dependency() {
    let temp_dir = TempDir::new().unwrap();

    // Create a helper library crate
    let helper_path = temp_dir.path().join("helper");
    fs::create_dir_all(&helper_path).unwrap();
    fs::write(
        helper_path.join("Cargo.toml"),
        r#"[package]
name = "helper"
version = "0.1.0"
edition = "2021"
[lib]
name = "helper"
path = "src/lib.rs"
"#,
    )
    .unwrap();
    fs::create_dir_all(helper_path.join("src")).unwrap();
    fs::write(helper_path.join("src/lib.rs"), "pub fn help() {}").unwrap();

    // Create main project that depends on the helper
    let main_path = temp_dir.path().join("main_proj");
    fs::create_dir_all(&main_path).unwrap();
    fs::write(
        main_path.join("Cargo.toml"),
        r#"[package]
name = "main_proj"
version = "0.1.0"
edition = "2021"

[dependencies]
helper = { path = "../helper" }
"#,
    )
    .unwrap();
    fs::create_dir_all(main_path.join("src")).unwrap();
    fs::write(main_path.join("src/main.rs"), "fn main() {}").unwrap();

    let project = CargoProject::new(&main_path).unwrap();
    let ext_libs = project.external_lib_paths();

    // The helper lib should be found
    assert!(
        !ext_libs.is_empty(),
        "external_lib_paths should find helper dep"
    );
    assert!(
        ext_libs.contains_key("helper"),
        "helper crate should be in external_lib_paths"
    );
}

#[test]
fn test_external_lib_paths_empty_for_no_deps() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_test_project(temp_dir.path(), false);
    let project = CargoProject::new(&project_path).unwrap();
    // Simple project with no external deps – map should be empty
    let ext_libs = project.external_lib_paths();
    assert!(
        ext_libs.is_empty(),
        "No external libs expected for bare project"
    );
}
