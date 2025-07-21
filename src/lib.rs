//! Rust Singler - A Rust code bundler for creating single-file applications
//!
//! This library provides functionality to bundle Rust projects into single source files,
//! combining multiple modules and dependencies into a single, self-contained file.

pub mod bundler;
pub mod cargo_project;
pub mod cli;
pub mod error;
pub mod file_manager;
pub mod transformer;

// Re-export main types for convenience
pub use bundler::Bundler;
pub use cargo_project::CargoProject;
pub use error::{BundlerError, Result};
pub use transformer::{CodeTransformer, TransformConfig};

use std::path::Path;

/// Creates a single-source-file version of a Cargo package.
///
/// This is a convenience function that uses the default bundler configuration.
/// For more control over the bundling process, use `Bundler::new()` directly.
///
/// # Arguments
///
/// * `package_path` - Path to the Cargo project directory
///
/// # Returns
///
/// A string containing the bundled source code
///
/// # Errors
///
/// Returns a `BundlerError` if the project cannot be bundled due to:
/// - Invalid project structure
/// - Missing files
/// - Parsing errors
/// - IO errors
///
/// # Example
///
/// ```rust,no_run
/// use rust_singler::bundle;
///
/// let bundled_code = bundle("./my_project").unwrap();
/// println!("{}", bundled_code);
/// ```
pub fn bundle<P: AsRef<Path>>(package_path: P) -> Result<String> {
    let bundler = Bundler::new();
    bundler.bundle(package_path)
}
