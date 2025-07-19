//! Configuration for code compression options

/// Configuration options for the compression process
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Whether to preserve string literal formatting
    pub preserve_string_formatting: bool,
    
    /// Whether to remove documentation comments
    pub remove_doc_comments: bool,
    
    /// Whether to remove test code
    pub remove_test_code: bool,
    
    /// Whether to output as a single line
    pub output_single_line: bool,
}

impl CompressionConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            preserve_string_formatting: true,
            remove_doc_comments: true,
            remove_test_code: true,
            output_single_line: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();
        assert!(config.preserve_string_formatting);
        assert!(config.remove_doc_comments);
        assert!(config.remove_test_code);
        assert!(config.output_single_line);
    }

    #[test]
    fn test_new_config() {
        let config = CompressionConfig::new();
        // Should be equivalent to default
        assert_eq!(config.preserve_string_formatting, true);
        assert_eq!(config.remove_doc_comments, true);
        assert_eq!(config.remove_test_code, true);
        assert_eq!(config.output_single_line, true);
    }
}
