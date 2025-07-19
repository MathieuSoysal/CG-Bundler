//! File processing implementation for reading and writing files

use std::fs;
use std::path::Path;
use crate::error::{ProcessingError, Result};
use crate::traits::FileProcessor;

/// Standard file processor implementation
pub struct StandardFileProcessor;

impl StandardFileProcessor {
    pub fn new() -> Self {
        Self
    }
    
    /// Ensure the output directory exists, creating it if necessary
    fn ensure_output_directory(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ProcessingError::OutputDirectoryError(
                        format!("Failed to create directory {}: {}", parent.display(), e)
                    ))?;
            }
        }
        Ok(())
    }
}

impl FileProcessor for StandardFileProcessor {
    fn read_file(&self, path: &Path) -> Result<String> {
        if !path.exists() {
            return Err(ProcessingError::FileNotFound(path.to_path_buf()));
        }
        
        fs::read_to_string(path)
            .map_err(|e| ProcessingError::IoError(e))
    }
    
    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        self.ensure_output_directory(path)?;
        
        fs::write(path, content)
            .map_err(|e| ProcessingError::IoError(e))
    }
}

impl Default for StandardFileProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_read_existing_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() {}";
        
        fs::write(&file_path, content).unwrap();
        
        let processor = StandardFileProcessor::new();
        let read_content = processor.read_file(&file_path)?;
        
        assert_eq!(read_content, content);
        Ok(())
    }
    
    #[test]
    fn test_read_nonexistent_file() {
        let processor = StandardFileProcessor::new();
        let result = processor.read_file(Path::new("/nonexistent/file.rs"));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessingError::FileNotFound(_)));
    }
    
    #[test]
    fn test_write_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.rs");
        let content = "fn test() { println!(\"hello\"); }";
        
        let processor = StandardFileProcessor::new();
        processor.write_file(&file_path, content)?;
        
        // Verify the file was written correctly
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
        
        Ok(())
    }
    
    #[test]
    fn test_write_file_with_directory_creation() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("nested").join("directory");
        let file_path = output_dir.join("output.rs");
        let content = "fn test() {}";
        
        // Directory doesn't exist yet
        assert!(!output_dir.exists());
        
        let processor = StandardFileProcessor::new();
        processor.write_file(&file_path, content)?;
        
        // Directory should be created and file should exist
        assert!(output_dir.exists());
        assert!(file_path.exists());
        
        let read_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(read_content, content);
        
        Ok(())
    }
    
    #[test]
    fn test_ensure_output_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a").join("b").join("c").join("file.rs");
        
        let processor = StandardFileProcessor::new();
        processor.ensure_output_directory(&nested_path)?;
        
        // Parent directories should exist
        assert!(nested_path.parent().unwrap().exists());
        
        Ok(())
    }
    
    #[test]
    fn test_ensure_output_directory_no_parent() -> Result<()> {
        let processor = StandardFileProcessor::new();
        // Root path has no parent
        let result = processor.ensure_output_directory(Path::new("/"));
        assert!(result.is_ok());
        
        Ok(())
    }
}
