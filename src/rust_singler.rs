//! Main application orchestrator for the rust-singler tool

use std::path::Path;
use crate::config::CompressionConfig;
use crate::error::{ProcessingError, Result};
use crate::traits::{FileDiscovery, CodeParser, CodeMinifier, FileProcessor, ErrorReporter, PerformanceTracker};

/// Main application orchestrator that coordinates all components
pub struct RustSingler {
    file_discovery: Box<dyn FileDiscovery>,
    code_parser: Box<dyn CodeParser>,
    code_minifier: Box<dyn CodeMinifier>,
    file_processor: Box<dyn FileProcessor>,
    error_reporter: Box<dyn ErrorReporter>,
    performance_tracker: Box<dyn PerformanceTracker>,
    config: CompressionConfig,
}

impl RustSingler {
    /// Create a new RustSingler instance with all dependencies
    pub fn new(
        file_discovery: Box<dyn FileDiscovery>,
        code_parser: Box<dyn CodeParser>,
        code_minifier: Box<dyn CodeMinifier>,
        file_processor: Box<dyn FileProcessor>,
        error_reporter: Box<dyn ErrorReporter>,
        performance_tracker: Box<dyn PerformanceTracker>,
        config: CompressionConfig,
    ) -> Self {
        Self {
            file_discovery,
            code_parser,
            code_minifier,
            file_processor,
            error_reporter,
            performance_tracker,
            config,
        }
    }
    
    /// Compress an entire directory of Rust files
    pub fn compress_directory(&mut self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.performance_tracker.start_timer("total_compression");
        self.performance_tracker.start_timer("file_discovery");
        
        let rust_files = self.file_discovery.find_rust_files(input_path)
            .map_err(|e| {
                self.error_reporter.report_error(&e);
                e
            })?;
        
        self.performance_tracker.end_timer("file_discovery");
        
        if rust_files.is_empty() {
            let error = ProcessingError::FileNotFound(input_path.to_path_buf());
            self.error_reporter.report_error(&error);
            return Err(error);
        }
        
        println!("📁 Found {} Rust files to process", rust_files.len());
        
        self.performance_tracker.start_timer("processing_all_files");
        
        let mut all_compressed_code = String::new();
        let mut processed_count = 0;
        
        for file_path in &rust_files {
            match self.process_single_file(file_path) {
                Ok(compressed) => {
                    if !all_compressed_code.is_empty() {
                        all_compressed_code.push(' ');
                    }
                    all_compressed_code.push_str(&compressed);
                    processed_count += 1;
                }
                Err(e) => {
                    self.error_reporter.report_error(&e);
                    // Continue processing other files
                }
            }
        }
        
        self.performance_tracker.end_timer("processing_all_files");
        
        if processed_count == 0 {
            let error = ProcessingError::CompressionError("No files were successfully processed".to_string());
            self.error_reporter.report_error(&error);
            return Err(error);
        }
        
        self.performance_tracker.start_timer("writing_output");
        
        self.file_processor.write_file(output_path, &all_compressed_code)
            .map_err(|e| {
                self.error_reporter.report_error(&e);
                e
            })?;
        
        self.performance_tracker.end_timer("writing_output");
        self.performance_tracker.end_timer("total_compression");
        
        println!("✅ Successfully compressed {} files to {}", 
                processed_count, output_path.display());
        
        self.performance_tracker.report_metrics();
        
        Ok(())
    }
    
    /// Compress a single Rust file
    pub fn compress_file(&mut self, input_file: &Path, output_file: &Path) -> Result<()> {
        self.performance_tracker.start_timer("single_file_compression");
        
        let compressed_code = self.process_single_file(input_file)
            .map_err(|e| {
                self.error_reporter.report_error(&e);
                e
            })?;
        
        self.file_processor.write_file(output_file, &compressed_code)
            .map_err(|e| {
                self.error_reporter.report_error(&e);
                e
            })?;
        
        self.performance_tracker.end_timer("single_file_compression");
        
        println!("✅ Successfully compressed {} to {}", 
                input_file.display(), output_file.display());
        
        self.performance_tracker.report_metrics();
        
        Ok(())
    }
    
    /// Process a single file and return the compressed code
    fn process_single_file(&mut self, file_path: &Path) -> Result<String> {
        self.performance_tracker.start_timer("read_file");
        
        let content = self.file_processor.read_file(file_path)?;
        
        self.performance_tracker.end_timer("read_file");
        self.performance_tracker.start_timer("parse_file");
        
        let mut syntax_tree = self.code_parser.parse(&content)?;
        
        self.performance_tracker.end_timer("parse_file");
        self.performance_tracker.start_timer("remove_unwanted");
        
        self.code_parser.remove_unwanted_elements(&mut syntax_tree)?;
        
        self.performance_tracker.end_timer("remove_unwanted");
        self.performance_tracker.start_timer("minify_code");
        
        let compressed = self.code_minifier.minify(&syntax_tree)?;
        
        self.performance_tracker.end_timer("minify_code");
        
        Ok(compressed)
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: CompressionConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::RecursiveFileDiscovery;
    use crate::parser::SynCodeParser;
    use crate::minifier::WhitespaceMinifier;
    use crate::file_processor::StandardFileProcessor;
    use crate::reporter::SilentErrorReporter;
    use crate::performance::NoOpPerformanceTracker;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_singler() -> RustSingler {
        RustSingler::new(
            Box::new(RecursiveFileDiscovery::new()),
            Box::new(SynCodeParser::new()),
            Box::new(WhitespaceMinifier::new()),
            Box::new(StandardFileProcessor::new()),
            Box::new(SilentErrorReporter),
            Box::new(NoOpPerformanceTracker),
            CompressionConfig::default(),
        )
    }

    #[test]
    fn test_compress_single_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("input.rs");
        let output_file = temp_dir.path().join("output.rs");
        
        let code = r#"
            /// This is a doc comment
            fn main() {
                println!("Hello, world!");
            }
            
            #[test]
            fn test_something() {
                assert_eq!(1, 1);
            }
        "#;
        
        fs::write(&input_file, code).unwrap();
        
        let mut singler = create_test_singler();
        singler.compress_file(&input_file, &output_file)?;
        
        let output = fs::read_to_string(&output_file).unwrap();
        
        // Should contain main function but not test function or doc comments
        assert!(output.contains("fn main"));
        assert!(output.contains("println!"));
        assert!(!output.contains("test_something"));
        assert!(!output.contains("This is a doc comment"));
        
        Ok(())
    }
    
    #[test]
    fn test_compress_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let output_file = temp_dir.path().join("compressed.rs");
        
        fs::create_dir(&src_dir).unwrap();
        
        let main_rs = src_dir.join("main.rs");
        let lib_rs = src_dir.join("lib.rs");
        
        fs::write(&main_rs, "fn main() { println!(\"main\"); }").unwrap();
        fs::write(&lib_rs, "pub fn lib_func() { println!(\"lib\"); }").unwrap();
        
        let mut singler = create_test_singler();
        singler.compress_directory(&src_dir, &output_file)?;
        
        let output = fs::read_to_string(&output_file).unwrap();
        
        // Should contain both functions
        assert!(output.contains("fn main"));
        assert!(output.contains("pub fn lib_func"));
        
        Ok(())
    }
    
    #[test]
    fn test_compress_nonexistent_file() {
        let mut singler = create_test_singler();
        let result = singler.compress_file(
            Path::new("/nonexistent.rs"), 
            Path::new("/output.rs")
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessingError::FileNotFound(_)));
    }
    
    #[test]
    fn test_compress_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();
        
        let mut singler = create_test_singler();
        let result = singler.compress_directory(&empty_dir, &temp_dir.path().join("output.rs"));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessingError::FileNotFound(_)));
    }
    
    #[test]
    fn test_process_single_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.rs");
        
        let code = r#"
            fn test() {
                let x = 42;
                println!("{}", x);
            }
        "#;
        
        fs::write(&input_file, code).unwrap();
        
        let mut singler = create_test_singler();
        let result = singler.process_single_file(&input_file)?;
        
        assert!(result.contains("fn test"));
        assert!(result.contains("let x"));
        assert!(result.contains("42"));
        assert!(!result.contains('\n')); // Should be minified
        
        Ok(())
    }
    
    #[test]
    fn test_config_management() {
        let mut singler = create_test_singler();
        
        let original_config = singler.config().clone();
        assert!(original_config.remove_test_code);
        
        let mut new_config = CompressionConfig::default();
        new_config.remove_test_code = false;
        
        singler.set_config(new_config);
        assert!(!singler.config().remove_test_code);
    }
}
