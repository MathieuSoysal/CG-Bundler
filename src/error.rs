//! Error types for the rust-singler application

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during code processing
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Invalid input path: {0}")]
    InvalidPath(PathBuf),
    
    #[error("Output directory creation failed: {0}")]
    OutputDirectoryError(String),
}

pub type Result<T> = std::result::Result<T, ProcessingError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_error_display() {
        let err = ProcessingError::FileNotFound(PathBuf::from("test.rs"));
        assert_eq!(err.to_string(), "File not found: test.rs");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ProcessingError::ParseError("Invalid syntax".to_string());
        assert_eq!(err.to_string(), "Parse error: Invalid syntax");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let err = ProcessingError::from(io_err);
        assert!(matches!(err, ProcessingError::IoError(_)));
    }
}
