//! File discovery implementation for finding Rust source files

use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use crate::error::{ProcessingError, Result};
use crate::traits::FileDiscovery;

/// Recursive file discovery implementation
pub struct RecursiveFileDiscovery;

impl RecursiveFileDiscovery {
    pub fn new() -> Self {
        Self
    }
    
    /// Check if a path represents a Rust source file
    fn is_rust_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "rs")
            .unwrap_or(false)
    }
    
    /// Check if a directory should be skipped during traversal
    fn should_skip_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            match name {
                "target" | ".git" | "node_modules" | ".cargo" => true,
                _ if name.starts_with('.') => true,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Filter function for walkdir entries
    #[allow(dead_code)]
    fn should_process_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        
        if path.is_dir() {
            !self.should_skip_directory(path)
        } else {
            self.is_rust_file(path)
        }
    }
}

impl FileDiscovery for RecursiveFileDiscovery {
    fn find_rust_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        if !path.exists() {
            return Err(ProcessingError::FileNotFound(path.to_path_buf()));
        }
        
        let mut rust_files = Vec::new();
        
        if path.is_file() {
            if self.is_rust_file(path) {
                rust_files.push(path.to_path_buf());
            }
            return Ok(rust_files);
        }
        
        // Handle directory traversal with proper filtering
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                // Allow the root directory
                if e.path() == path {
                    return true;
                }
                
                // For directories, check if we should skip them
                if e.path().is_dir() {
                    return !self.should_skip_directory(e.path());
                }
                
                // For files, allow all files (we'll filter rust files later)
                true
            })
        {
            let entry = entry.map_err(|e| ProcessingError::IoError(e.into()))?;
            let entry_path = entry.path();
            
            if entry_path.is_file() && self.is_rust_file(entry_path) {
                rust_files.push(entry_path.to_path_buf());
            }
        }
        
        rust_files.sort();
        Ok(rust_files)
    }
}

impl Default for RecursiveFileDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_rust_file() {
        let discovery = RecursiveFileDiscovery::new();
        
        assert!(discovery.is_rust_file(Path::new("main.rs")));
        assert!(discovery.is_rust_file(Path::new("lib.rs")));
        assert!(discovery.is_rust_file(Path::new("foo/bar.rs")));
        
        assert!(!discovery.is_rust_file(Path::new("main.txt")));
        assert!(!discovery.is_rust_file(Path::new("Cargo.toml")));
        assert!(!discovery.is_rust_file(Path::new("README.md")));
    }
    
    #[test]
    fn test_should_skip_directory() {
        let discovery = RecursiveFileDiscovery::new();
        
        assert!(discovery.should_skip_directory(Path::new("target")));
        assert!(discovery.should_skip_directory(Path::new(".git")));
        assert!(discovery.should_skip_directory(Path::new(".cargo")));
        assert!(discovery.should_skip_directory(Path::new(".hidden")));
        
        assert!(!discovery.should_skip_directory(Path::new("src")));
        assert!(!discovery.should_skip_directory(Path::new("tests")));
        assert!(!discovery.should_skip_directory(Path::new("examples")));
    }
    
    #[test]
    fn test_find_single_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        fs::write(&rust_file, "fn main() {}")?;
        
        let discovery = RecursiveFileDiscovery::new();
        let files = discovery.find_rust_files(&rust_file)?;
        
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], rust_file);
        
        Ok(())
    }
    
    #[test]
    fn test_find_files_in_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;
        
        let main_rs = src_dir.join("main.rs");
        let lib_rs = src_dir.join("lib.rs");
        let txt_file = src_dir.join("readme.txt");
        
        fs::write(&main_rs, "fn main() {}")?;
        fs::write(&lib_rs, "pub fn lib() {}")?;
        fs::write(&txt_file, "Not a rust file")?;
        
        let discovery = RecursiveFileDiscovery::new();
        let files = discovery.find_rust_files(&src_dir)?;
        
        assert_eq!(files.len(), 2);
        assert!(files.contains(&lib_rs));
        assert!(files.contains(&main_rs));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "readme.txt"));
        
        Ok(())
    }
    
    #[test]
    fn test_skip_target_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir)?;
        
        let target_file = target_dir.join("should_skip.rs");
        fs::write(&target_file, "fn should_skip() {}")?;
        
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir)?;
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {}")?;
        
        let discovery = RecursiveFileDiscovery::new();
        let files = discovery.find_rust_files(temp_dir.path())?;
        
        // Check that we found files and none are from target directory
        assert!(files.len() >= 1, "Should find at least the main.rs file");
        assert!(files.iter().any(|f| f.file_name().unwrap() == "main.rs"), "Should find main.rs");
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "should_skip.rs"), "Should not find should_skip.rs from target dir");
        
        Ok(())
    }
    
    #[test]
    fn test_nonexistent_path() {
        let discovery = RecursiveFileDiscovery::new();
        let result = discovery.find_rust_files(Path::new("/nonexistent/path"));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessingError::FileNotFound(_)));
    }
}
