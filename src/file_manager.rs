use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{BundlerError, Result};

/// Utility struct for file operations
pub struct FileManager;

impl FileManager {
    /// Read file contents into a string
    ///
    /// # Errors
    /// Returns an error if the file cannot be read
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
    /// Returns (`base_path_for_submodules`, `file_content`)
    ///
    /// # Errors
    /// Returns an error if the module file cannot be found or read
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
