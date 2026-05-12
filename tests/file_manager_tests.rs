use cg_bundler::error::BundlerError;
use cg_bundler::file_manager::FileManager;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_read_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let content = "Hello, World!";

    fs::write(&file_path, content).unwrap();

    let result = FileManager::read_file(&file_path).unwrap();
    assert_eq!(result, content);
}

#[test]
fn test_read_nonexistent_file() {
    let result = FileManager::read_file("/nonexistent/file.txt");
    assert!(result.is_err());
}

#[test]
fn test_file_exists() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    assert!(!FileManager::file_exists(&file_path));

    fs::write(&file_path, "content").unwrap();
    assert!(FileManager::file_exists(&file_path));
}

#[test]
fn test_try_read_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // File doesn't exist
    assert!(FileManager::try_read_file(&file_path).is_none());

    // File exists
    let content = "test content";
    fs::write(&file_path, content).unwrap();
    assert_eq!(
        FileManager::try_read_file(&file_path),
        Some(content.to_string())
    );
}

#[test]
fn test_find_module_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let result = FileManager::find_module_file(temp_dir.path(), "nonexistent");
    assert!(result.is_err());
    match result.unwrap_err() {
        BundlerError::ProjectStructure { message } => {
            assert!(message.contains("nonexistent"));
        }
        _ => panic!("Expected ProjectStructure error"),
    }
}

#[test]
fn test_find_module_file_via_rs_file() {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("mymod.rs"), "pub fn in_mod() {}").unwrap();
    let (base, content) = FileManager::find_module_file(temp_dir.path(), "mymod").unwrap();
    assert!(content.contains("in_mod"));
    assert_eq!(base, temp_dir.path().join("mymod"));
}

#[test]
fn test_find_module_file_via_mod_rs() {
    let temp_dir = TempDir::new().unwrap();
    let mod_dir = temp_dir.path().join("mymod");
    fs::create_dir_all(&mod_dir).unwrap();
    fs::write(mod_dir.join("mod.rs"), "pub fn in_mod_rs() {}").unwrap();
    let (base, content) = FileManager::find_module_file(temp_dir.path(), "mymod").unwrap();
    assert!(content.contains("in_mod_rs"));
    assert_eq!(base, mod_dir);
}
