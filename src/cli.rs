//! Command-line interface definition for `cg-bundler`.
//!
//! Keeps argument parsing, defaults, and small accessors in a single,
//! self-contained module so the binary entry point stays focused on
//! orchestration rather than option plumbing.

use std::path::PathBuf;

use clap::Parser;

use cg_bundler::TransformConfig;

/// A Rust code bundler that combines multiple source files into a single file
#[derive(Parser, Debug)]
#[command(name = "cg-bundler")]
#[command(about = "Bundle Rust projects into single files")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "CG Bundler Contributors")]
#[command(
    long_about = "A Rust code bundler that combines multiple source files into a single file.\nBy default, bundles the current directory or the specified project path.\n\n🐛 Found a bug or need help?\n   Report issues: https://github.com/MathieuSoysal/CG-Bundler/issues/new\n\n📖 Documentation:\n   https://docs.rs/cg-bundler"
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Path to the Cargo project directory (defaults to current directory)
    #[arg(
        value_name = "PROJECT_PATH",
        help = "Path to bundle (defaults to current directory)"
    )]
    pub project_path: Option<PathBuf>,

    /// Output file path (stdout if not specified)
    #[arg(short, long, value_name = "FILE", help = "Output file path")]
    pub output: Option<PathBuf>,

    /// Keep test code in the bundled output
    #[arg(long, help = "Keep test code in the bundled output")]
    pub keep_tests: bool,

    /// Keep documentation comments in the bundled output
    #[arg(long, help = "Keep documentation comments")]
    pub keep_docs: bool,

    /// Disable module expansion (keep module declarations)
    #[arg(long, help = "Disable module expansion")]
    pub no_expand_modules: bool,

    /// Pretty print the output (format with rustfmt if available)
    #[arg(long, help = "Pretty print the output")]
    pub pretty: bool,

    /// Minify the output to a single line
    #[arg(short, long, help = "Minify the output")]
    pub minify: bool,

    /// Aggressive minify with whitespace replacements (implies -m)
    #[arg(long, help = "Aggressive minify")]
    pub m2: bool,

    /// Verbose output
    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,

    /// Validate that the project can be bundled without errors (instead of bundling)
    #[arg(long, help = "Validate that the project can be bundled without errors")]
    pub validate: bool,

    /// Show information about the Cargo project structure (instead of bundling)
    #[arg(long, help = "Show information about the Cargo project structure")]
    pub info: bool,

    /// Watch for file changes and rebuild automatically
    #[arg(short, long, help = "Watch for file changes and rebuild automatically")]
    pub watch: bool,

    /// Source directory to watch (default: src)
    #[arg(long, default_value = "src", help = "Source directory to watch")]
    pub src_dir: String,

    /// Debounce delay in milliseconds (default: 500)
    #[arg(long, default_value = "500", help = "Debounce delay in milliseconds")]
    pub debounce: u64,
}

impl Cli {
    /// Get the effective project path, using current directory as default.
    #[must_use]
    pub fn get_project_path(&self) -> PathBuf {
        self.project_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Whether verbose mode is enabled.
    #[must_use]
    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Build the [`TransformConfig`] expected by the bundler from CLI flags.
    #[must_use]
    pub const fn get_transform_config(&self) -> TransformConfig {
        TransformConfig {
            remove_tests: !self.keep_tests,
            remove_docs: !self.keep_docs,
            expand_modules: !self.no_expand_modules,
            minify: self.minify || self.m2,
            aggressive_minify: self.m2,
        }
    }

    /// Optional output file path.
    #[must_use]
    pub const fn get_output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Whether `rustfmt` pretty-printing is requested.
    #[must_use]
    pub const fn is_pretty(&self) -> bool {
        self.pretty
    }

    /// Whether minification (basic or aggressive) is requested.
    #[must_use]
    pub const fn is_minify(&self) -> bool {
        self.minify || self.m2
    }

    /// Whether aggressive minification is requested.
    #[must_use]
    pub const fn is_aggressive_minify(&self) -> bool {
        self.m2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_project_path_default() {
        let cli = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert_eq!(cli.get_project_path(), PathBuf::from("."));
    }

    #[test]
    fn test_get_project_path_custom() {
        let cli = Cli::try_parse_from(["cg-bundler", "/tmp/myproj"]).unwrap();
        assert_eq!(cli.get_project_path(), PathBuf::from("/tmp/myproj"));
    }

    #[test]
    fn test_is_verbose_flag() {
        let cli = Cli::try_parse_from(["cg-bundler", "--verbose"]).unwrap();
        assert!(cli.is_verbose());

        let cli2 = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli2.is_verbose());
    }

    #[test]
    fn test_is_pretty() {
        let cli = Cli::try_parse_from(["cg-bundler", "--pretty"]).unwrap();
        assert!(cli.is_pretty());

        let cli2 = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli2.is_pretty());
    }

    #[test]
    fn test_is_minify_and_aggressive() {
        let cli_m = Cli::try_parse_from(["cg-bundler", "--minify"]).unwrap();
        assert!(cli_m.is_minify());
        assert!(!cli_m.is_aggressive_minify());

        let cli_m2 = Cli::try_parse_from(["cg-bundler", "--m2"]).unwrap();
        assert!(cli_m2.is_minify());
        assert!(cli_m2.is_aggressive_minify());

        let cli_none = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(!cli_none.is_minify());
        assert!(!cli_none.is_aggressive_minify());
    }

    #[test]
    fn test_get_output() {
        let cli_none = Cli::try_parse_from(["cg-bundler"]).unwrap();
        assert!(cli_none.get_output().is_none());

        let cli_out = Cli::try_parse_from(["cg-bundler", "-o", "out.rs"]).unwrap();
        assert!(cli_out.get_output().is_some());
        assert_eq!(cli_out.get_output().unwrap(), &PathBuf::from("out.rs"));
    }

    #[test]
    fn test_get_transform_config_flags() {
        let cli = Cli::try_parse_from(["cg-bundler", "--keep-tests", "--keep-docs"]).unwrap();
        let config = cli.get_transform_config();
        assert!(!config.remove_tests);
        assert!(!config.remove_docs);
        assert!(config.expand_modules);
        assert!(!config.minify);
        assert!(!config.aggressive_minify);

        let cli_m2 = Cli::try_parse_from(["cg-bundler", "--m2"]).unwrap();
        let config_m2 = cli_m2.get_transform_config();
        assert!(config_m2.aggressive_minify);
        assert!(config_m2.minify);

        let cli_no = Cli::try_parse_from(["cg-bundler", "--no-expand-modules"]).unwrap();
        let config_no = cli_no.get_transform_config();
        assert!(!config_no.expand_modules);
    }
}
