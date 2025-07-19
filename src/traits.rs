//! Core traits and interfaces for the rust-singler application

use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::syntax_tree::SyntaxTree;

/// Trait for discovering Rust files in a directory structure
pub trait FileDiscovery {
    /// Find all Rust files starting from the given path
    fn find_rust_files(&self, path: &Path) -> Result<Vec<PathBuf>>;
}

/// Trait for parsing Rust code and removing unwanted elements
pub trait CodeParser {
    /// Parse Rust source code into a syntax tree
    fn parse(&self, content: &str) -> Result<SyntaxTree>;
    
    /// Remove unwanted elements from the syntax tree
    fn remove_unwanted_elements(&self, tree: &mut SyntaxTree) -> Result<()>;
}

/// Trait for minifying code into compressed format
pub trait CodeMinifier {
    /// Minify a syntax tree into compressed code
    fn minify(&self, tree: &SyntaxTree) -> Result<String>;
    
    /// Compress code to a single line
    fn compress_to_single_line(&self, code: &str) -> Result<String>;
}

/// Trait for file system operations
pub trait FileProcessor {
    /// Read content from a file
    fn read_file(&self, path: &Path) -> Result<String>;
    
    /// Write content to a file
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
}

/// Trait for reporting errors to users
pub trait ErrorReporter {
    /// Report an error to the user
    fn report_error(&self, error: &crate::error::ProcessingError);
    
    /// Format an error message for display
    fn format_error_message(&self, error: &crate::error::ProcessingError) -> String;
}

/// Trait for performance tracking
pub trait PerformanceTracker {
    /// Start timing an operation
    fn start_timer(&mut self, operation: &str);
    
    /// End timing an operation
    fn end_timer(&mut self, operation: &str);
    
    /// Report collected metrics
    fn report_metrics(&self);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock implementations for testing
    struct MockFileDiscovery;
    impl FileDiscovery for MockFileDiscovery {
        fn find_rust_files(&self, _path: &Path) -> Result<Vec<PathBuf>> {
            Ok(vec![PathBuf::from("test.rs")])
        }
    }
    
    #[test]
    fn test_mock_file_discovery() {
        let discovery = MockFileDiscovery;
        let files = discovery.find_rust_files(Path::new("/test")).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], PathBuf::from("test.rs"));
    }
}
