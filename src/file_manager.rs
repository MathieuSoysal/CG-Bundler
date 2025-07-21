use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{BundlerError, Result};

/// Utility struct for file operations
pub struct FileManager;

impl FileManager {
    /// Read file contents into a string
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        let mut buf = String::new();

        let mut file = File::open(path).map_err(|e| BundlerError::Io {
            source: e,
            path: Some(path.to_path_buf()),
        })?;

        file.read_to_string(&mut buf)
            .map_err(|e| BundlerError::Io {
                source: e,
                path: Some(path.to_path_buf()),
            })?;

        Ok(buf)
    }

    /// Check if a file exists
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Try to read a file, returning None if it doesn't exist
    pub fn try_read_file<P: AsRef<Path>>(path: P) -> Option<String> {
        Self::read_file(path).ok()
    }

    /// Find a module file by trying different possible locations
    /// Returns (base_path_for_submodules, file_content)
    pub fn find_module_file(base_path: &Path, module_name: &str) -> Result<(PathBuf, String)> {
        let possible_locations = vec![
            // Look for module_name.rs in base_path, submodules will be in base_path/module_name/
            (
                base_path.to_path_buf(),
                format!("{module_name}.rs"),
                base_path.join(module_name),
            ),
            // Look for mod.rs in base_path/module_name/, submodules will be in base_path/module_name/
            (
                base_path.join(module_name),
                "mod.rs".to_string(),
                base_path.join(module_name),
            ),
        ];

        for (file_base, file_name, submodule_base) in possible_locations {
            let full_path = file_base.join(&file_name);
            if let Ok(content) = Self::read_file(&full_path) {
                return Ok((submodule_base, content));
            }
        }

        Err(BundlerError::ProjectStructure {
            message: format!("Module '{module_name}' not found in expected locations"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
