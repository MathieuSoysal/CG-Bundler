//! Error reporting implementation for user-friendly error messages

use colored::Colorize;
use crate::error::ProcessingError;
use crate::traits::ErrorReporter;

/// Console-based error reporter with colored output
pub struct ConsoleErrorReporter {
    use_colors: bool,
}

impl ConsoleErrorReporter {
    pub fn new() -> Self {
        Self {
            use_colors: true,
        }
    }
    
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }
    
    /// Format text with colors if enabled
    fn format_with_colors(&self, message: &str, color_type: ColorType) -> String {
        if !self.use_colors {
            return message.to_string();
        }
        
        match color_type {
            ColorType::Error => message.red().bold().to_string(),
            ColorType::Warning => message.yellow().bold().to_string(),
            ColorType::Info => message.blue().to_string(),
            ColorType::Success => message.green().to_string(),
            ColorType::Path => message.cyan().to_string(),
        }
    }
}

impl ErrorReporter for ConsoleErrorReporter {
    fn report_error(&self, error: &ProcessingError) {
        let message = self.format_error_message(error);
        eprintln!("{}", message);
    }
    
    fn format_error_message(&self, error: &ProcessingError) -> String {
        match error {
            ProcessingError::FileNotFound(path) => {
                format!(
                    "{}: File not found: {}",
                    self.format_with_colors("Error", ColorType::Error),
                    self.format_with_colors(&path.display().to_string(), ColorType::Path)
                )
            }
            ProcessingError::ParseError(msg) => {
                format!(
                    "{}: Failed to parse Rust code: {}",
                    self.format_with_colors("Parse Error", ColorType::Error),
                    msg
                )
            }
            ProcessingError::IoError(io_err) => {
                format!(
                    "{}: I/O operation failed: {}",
                    self.format_with_colors("I/O Error", ColorType::Error),
                    io_err
                )
            }
            ProcessingError::CompressionError(msg) => {
                format!(
                    "{}: Code compression failed: {}",
                    self.format_with_colors("Compression Error", ColorType::Error),
                    msg
                )
            }
            ProcessingError::InvalidPath(path) => {
                format!(
                    "{}: Invalid path specified: {}",
                    self.format_with_colors("Invalid Path", ColorType::Error),
                    self.format_with_colors(&path.display().to_string(), ColorType::Path)
                )
            }
            ProcessingError::OutputDirectoryError(msg) => {
                format!(
                    "{}: Failed to create output directory: {}",
                    self.format_with_colors("Directory Error", ColorType::Error),
                    msg
                )
            }
        }
    }
}

impl Default for ConsoleErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum ColorType {
    Error,
    Warning,
    Info,
    Success,
    Path,
}

/// Silent error reporter for testing
pub struct SilentErrorReporter;

impl ErrorReporter for SilentErrorReporter {
    fn report_error(&self, _error: &ProcessingError) {
        // Do nothing - used for testing
    }
    
    fn format_error_message(&self, error: &ProcessingError) -> String {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_file_not_found_error() {
        let reporter = ConsoleErrorReporter::new().with_colors(false);
        let error = ProcessingError::FileNotFound(PathBuf::from("test.rs"));
        
        let message = reporter.format_error_message(&error);
        assert!(message.contains("Error: File not found"));
        assert!(message.contains("test.rs"));
    }
    
    #[test]
    fn test_format_parse_error() {
        let reporter = ConsoleErrorReporter::new().with_colors(false);
        let error = ProcessingError::ParseError("Invalid syntax".to_string());
        
        let message = reporter.format_error_message(&error);
        assert!(message.contains("Parse Error"));
        assert!(message.contains("Invalid syntax"));
    }
    
    #[test]
    fn test_format_io_error() {
        let reporter = ConsoleErrorReporter::new().with_colors(false);
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let error = ProcessingError::IoError(io_err);
        
        let message = reporter.format_error_message(&error);
        assert!(message.contains("I/O Error"));
        assert!(message.contains("Access denied"));
    }
    
    #[test]
    fn test_format_compression_error() {
        let reporter = ConsoleErrorReporter::new().with_colors(false);
        let error = ProcessingError::CompressionError("Failed to minify".to_string());
        
        let message = reporter.format_error_message(&error);
        assert!(message.contains("Compression Error"));
        assert!(message.contains("Failed to minify"));
    }
    
    #[test]
    fn test_format_with_colors_disabled() {
        let reporter = ConsoleErrorReporter::new().with_colors(false);
        let formatted = reporter.format_with_colors("test message", ColorType::Error);
        assert_eq!(formatted, "test message");
    }
    
    #[test]
    fn test_format_with_colors_enabled() {
        let reporter = ConsoleErrorReporter::new().with_colors(true);
        let formatted = reporter.format_with_colors("test message", ColorType::Error);
        // Should contain ANSI color codes
        assert!(formatted.len() > "test message".len());
    }
    
    #[test]
    fn test_silent_error_reporter() {
        let reporter = SilentErrorReporter;
        let error = ProcessingError::FileNotFound(PathBuf::from("test.rs"));
        
        // Should not panic
        reporter.report_error(&error);
        
        let message = reporter.format_error_message(&error);
        assert_eq!(message, "File not found: test.rs");
    }
}
