use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::transformer::TransformConfig;

/// A Rust code bundler that combines multiple source files into a single file
#[derive(Parser, Debug)]
#[command(name = "rust-singler")]
#[command(about = "A Rust code bundler for creating single-file applications")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Rust Singler Contributors")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bundle a Rust project into a single source file
    Bundle {
        /// Path to the Cargo project directory
        #[arg(value_name = "PROJECT_PATH")]
        project_path: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Keep test code in the bundled output
        #[arg(long)]
        keep_tests: bool,

        /// Keep documentation comments in the bundled output
        #[arg(long)]
        keep_docs: bool,

        /// Disable module expansion (keep module declarations)
        #[arg(long)]
        no_expand_modules: bool,

        /// Pretty print the output (format with rustfmt if available)
        #[arg(long)]
        pretty: bool,

        /// Minify the output to a single line
        #[arg(short, long)]
        minify: bool,

        /// Aggressive minify with whitespace replacements (implies -m)
        #[arg(long)]
        m2: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate that a project can be bundled without errors
    Validate {
        /// Path to the Cargo project directory
        #[arg(value_name = "PROJECT_PATH")]
        project_path: PathBuf,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show information about a Cargo project structure
    Info {
        /// Path to the Cargo project directory
        #[arg(value_name = "PROJECT_PATH")]
        project_path: PathBuf,
    },
}

impl Commands {
    /// Get the project path from any command
    pub fn project_path(&self) -> &PathBuf {
        match self {
            Commands::Bundle { project_path, .. } => project_path,
            Commands::Validate { project_path, .. } => project_path,
            Commands::Info { project_path } => project_path,
        }
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        match self {
            Commands::Bundle { verbose, .. } => *verbose,
            Commands::Validate { verbose, .. } => *verbose,
            Commands::Info { .. } => false,
        }
    }

    /// Get transform configuration for bundle command
    pub fn get_transform_config(&self) -> Option<TransformConfig> {
        match self {
            Commands::Bundle {
                keep_tests,
                keep_docs,
                no_expand_modules,
                minify,
                m2,
                ..
            } => Some(TransformConfig {
                remove_tests: !keep_tests,
                remove_docs: !keep_docs,
                expand_modules: !no_expand_modules,
                minify: *minify || *m2, // m2 implies minify
                aggressive_minify: *m2,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verify() {
        // Verify the CLI definition is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_bundle_command_parsing() {
        let args = vec![
            "rust-singler",
            "bundle",
            "/path/to/project",
            "--output",
            "output.rs",
            "--keep-tests",
            "--verbose",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Bundle {
                project_path,
                output,
                keep_tests,
                verbose,
                ..
            } => {
                assert_eq!(project_path, PathBuf::from("/path/to/project"));
                assert_eq!(output, Some(PathBuf::from("output.rs")));
                assert!(keep_tests);
                assert!(verbose);
            }
            _ => panic!("Expected Bundle command"),
        }
    }

    #[test]
    fn test_validate_command_parsing() {
        let args = vec!["rust-singler", "validate", "/path/to/project", "--verbose"];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Validate {
                project_path,
                verbose,
            } => {
                assert_eq!(project_path, PathBuf::from("/path/to/project"));
                assert!(verbose);
            }
            _ => panic!("Expected Validate command"),
        }
    }

    #[test]
    fn test_info_command_parsing() {
        let args = vec!["rust-singler", "info", "/path/to/project"];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Info { project_path } => {
                assert_eq!(project_path, PathBuf::from("/path/to/project"));
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn test_get_transform_config() {
        let bundle_cmd = Commands::Bundle {
            project_path: PathBuf::from("/test"),
            output: None,
            keep_tests: true,
            keep_docs: false,
            no_expand_modules: true,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
        };

        let config = bundle_cmd.get_transform_config().unwrap();
        assert!(!config.remove_tests); // keep_tests = true
        assert!(config.remove_docs); // keep_docs = false
        assert!(!config.expand_modules); // no_expand_modules = true

        let validate_cmd = Commands::Validate {
            project_path: PathBuf::from("/test"),
            verbose: false,
        };

        assert!(validate_cmd.get_transform_config().is_none());
    }

    #[test]
    fn test_project_path_extraction() {
        let test_path = PathBuf::from("/test/path");

        let bundle_cmd = Commands::Bundle {
            project_path: test_path.clone(),
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
        };

        assert_eq!(bundle_cmd.project_path(), &test_path);
    }

    #[test]
    fn test_verbose_detection() {
        let verbose_bundle = Commands::Bundle {
            project_path: PathBuf::from("/test"),
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: true,
        };

        assert!(verbose_bundle.is_verbose());

        let non_verbose_bundle = Commands::Bundle {
            project_path: PathBuf::from("/test"),
            output: None,
            keep_tests: false,
            keep_docs: false,
            no_expand_modules: false,
            pretty: false,
            minify: false,
            m2: false,
            verbose: false,
        };

        assert!(!non_verbose_bundle.is_verbose());
    }
}
