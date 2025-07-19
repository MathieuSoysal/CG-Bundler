//! Command-line interface for the rust-singler application

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::config::CompressionConfig;
use crate::error::Result;
use crate::rust_singler::RustSingler;
use crate::discovery::RecursiveFileDiscovery;
use crate::parser::SynCodeParser;
use crate::minifier::WhitespaceMinifier;
use crate::file_processor::StandardFileProcessor;
use crate::reporter::ConsoleErrorReporter;
use crate::performance::MetricsCollector;

/// Command-line arguments for rust-singler
#[derive(Parser, Debug)]
#[command(name = "rust-singler")]
#[command(version = "0.1.0")]
#[command(about = "A Rust code compression tool that minifies Rust codebases into single-line format")]
#[command(long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,
    
    /// Disable performance metrics
    #[arg(long, global = true)]
    pub no_metrics: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compress an entire directory of Rust files
    Directory {
        /// Input directory path (should contain Rust files)
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path for the compressed code
        #[arg(short, long)]
        output: PathBuf,
        
        /// Preserve string literal formatting
        #[arg(long, default_value_t = true)]
        preserve_strings: bool,
        
        /// Keep documentation comments
        #[arg(long)]
        keep_docs: bool,
        
        /// Keep test code
        #[arg(long)]
        keep_tests: bool,
        
        /// Output as multiple lines instead of single line
        #[arg(long)]
        multiline: bool,
    },
    
    /// Compress a single Rust file
    File {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Preserve string literal formatting
        #[arg(long, default_value_t = true)]
        preserve_strings: bool,
        
        /// Keep documentation comments
        #[arg(long)]
        keep_docs: bool,
        
        /// Keep test code
        #[arg(long)]
        keep_tests: bool,
        
        /// Output as multiple lines instead of single line
        #[arg(long)]
        multiline: bool,
    },
}

/// CLI application wrapper
pub struct CliApplication;

impl CliApplication {
    pub fn new() -> Self {
        Self
    }
    
    /// Run the CLI application with the given arguments
    pub fn run(&self, args: CliArgs) -> Result<()> {
        self.validate_arguments(&args)?;
        
        let config = self.create_config_from_args(&args);
        let mut singler = self.create_rust_singler(config, &args);
        
        match args.command {
            Commands::Directory { ref input, ref output, .. } => {
                if args.verbose {
                    println!("🔍 Processing directory: {}", input.display());
                    println!("📝 Output file: {}", output.display());
                }
                singler.compress_directory(input, output)
            }
            Commands::File { ref input, ref output, .. } => {
                if args.verbose {
                    println!("🔍 Processing file: {}", input.display());
                    println!("📝 Output file: {}", output.display());
                }
                singler.compress_file(input, output)
            }
        }
    }
    
    /// Parse command-line arguments
    pub fn parse_arguments() -> CliArgs {
        CliArgs::parse()
    }
    
    /// Validate the provided arguments
    fn validate_arguments(&self, args: &CliArgs) -> Result<()> {
        match &args.command {
            Commands::Directory { input, .. } => {
                if !input.exists() {
                    return Err(crate::error::ProcessingError::InvalidPath(input.clone()));
                }
                if !input.is_dir() {
                    return Err(crate::error::ProcessingError::InvalidPath(input.clone()));
                }
            }
            Commands::File { input, .. } => {
                if !input.exists() {
                    return Err(crate::error::ProcessingError::FileNotFound(input.clone()));
                }
                if !input.is_file() {
                    return Err(crate::error::ProcessingError::InvalidPath(input.clone()));
                }
                if input.extension().and_then(|s| s.to_str()) != Some("rs") {
                    return Err(crate::error::ProcessingError::InvalidPath(input.clone()));
                }
            }
        }
        Ok(())
    }
    
    /// Create compression config from CLI arguments
    fn create_config_from_args(&self, args: &CliArgs) -> CompressionConfig {
        let (preserve_strings, keep_docs, keep_tests, multiline) = match &args.command {
            Commands::Directory { preserve_strings, keep_docs, keep_tests, multiline, .. } => {
                (*preserve_strings, *keep_docs, *keep_tests, *multiline)
            }
            Commands::File { preserve_strings, keep_docs, keep_tests, multiline, .. } => {
                (*preserve_strings, *keep_docs, *keep_tests, *multiline)
            }
        };
        
        CompressionConfig {
            preserve_string_formatting: preserve_strings,
            remove_doc_comments: !keep_docs,
            remove_test_code: !keep_tests,
            output_single_line: !multiline,
        }
    }
    
    /// Create a RustSingler instance with appropriate dependencies
    fn create_rust_singler(&self, config: CompressionConfig, args: &CliArgs) -> RustSingler {
        let file_discovery = Box::new(RecursiveFileDiscovery::new());
        let code_parser = Box::new(SynCodeParser::new());
        let code_minifier = Box::new(WhitespaceMinifier::new()
            .with_string_preservation(config.preserve_string_formatting));
        let file_processor = Box::new(StandardFileProcessor::new());
        let error_reporter = Box::new(ConsoleErrorReporter::new()
            .with_colors(!args.no_color));
        let performance_tracker = Box::new(MetricsCollector::new()
            .with_enabled(!args.no_metrics));
        
        RustSingler::new(
            file_discovery,
            code_parser,
            code_minifier,
            file_processor,
            error_reporter,
            performance_tracker,
            config,
        )
    }
}

impl Default for CliApplication {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_app() -> CliApplication {
        CliApplication::new()
    }

    #[test]
    fn test_validate_valid_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let args = CliArgs {
            command: Commands::Directory {
                input: src_dir,
                output: temp_dir.path().join("output.rs"),
                preserve_strings: true,
                keep_docs: false,
                keep_tests: false,
                multiline: false,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let app = create_test_app();
        let result = app.validate_arguments(&args);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_validate_nonexistent_directory() {
        let args = CliArgs {
            command: Commands::Directory {
                input: PathBuf::from("/nonexistent/directory"),
                output: PathBuf::from("output.rs"),
                preserve_strings: true,
                keep_docs: false,
                keep_tests: false,
                multiline: false,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let app = create_test_app();
        let result = app.validate_arguments(&args);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_valid_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.rs");
        fs::write(&input_file, "fn main() {}").unwrap();
        
        let args = CliArgs {
            command: Commands::File {
                input: input_file,
                output: temp_dir.path().join("output.rs"),
                preserve_strings: true,
                keep_docs: false,
                keep_tests: false,
                multiline: false,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let app = create_test_app();
        let result = app.validate_arguments(&args);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_validate_non_rust_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.path().join("test.txt");
        fs::write(&input_file, "not rust code").unwrap();
        
        let args = CliArgs {
            command: Commands::File {
                input: input_file,
                output: temp_dir.path().join("output.rs"),
                preserve_strings: true,
                keep_docs: false,
                keep_tests: false,
                multiline: false,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let app = create_test_app();
        let result = app.validate_arguments(&args);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_create_config_from_args() {
        let app = create_test_app();
        
        let args = CliArgs {
            command: Commands::File {
                input: PathBuf::from("test.rs"),
                output: PathBuf::from("output.rs"),
                preserve_strings: false,
                keep_docs: true,
                keep_tests: true,
                multiline: true,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let config = app.create_config_from_args(&args);
        
        assert!(!config.preserve_string_formatting);
        assert!(!config.remove_doc_comments); // keep_docs = true means remove_doc_comments = false
        assert!(!config.remove_test_code);    // keep_tests = true means remove_test_code = false
        assert!(!config.output_single_line);  // multiline = true means output_single_line = false
    }
    
    #[test]
    fn test_create_config_default_values() {
        let app = create_test_app();
        
        let args = CliArgs {
            command: Commands::Directory {
                input: PathBuf::from("src"),
                output: PathBuf::from("output.rs"),
                preserve_strings: true,
                keep_docs: false,
                keep_tests: false,
                multiline: false,
            },
            verbose: false,
            no_color: false,
            no_metrics: false,
        };
        
        let config = app.create_config_from_args(&args);
        
        assert!(config.preserve_string_formatting);
        assert!(config.remove_doc_comments);
        assert!(config.remove_test_code);
        assert!(config.output_single_line);
    }
}
