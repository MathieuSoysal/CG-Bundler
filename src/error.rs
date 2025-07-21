use std::fmt;
use std::io;
use std::path::PathBuf;

/// Custom error types for the cg-bundler application
#[derive(Debug)]
pub enum BundlerError {
    /// IO related errors (file reading, writing, etc.)
    Io {
        source: io::Error,
        path: Option<PathBuf>,
    },
    /// Cargo metadata related errors
    CargoMetadata {
        message: String,
        source: Option<cargo_metadata::Error>,
    },
    /// Syntax parsing errors
    Parsing {
        message: String,
        file_path: Option<PathBuf>,
    },
    /// Project structure related errors
    ProjectStructure { message: String },
    /// Multiple binary targets found (not supported)
    MultipleBinaryTargets { target_count: usize },
    /// No binary target found
    NoBinaryTarget,
    /// Multiple library targets found (not supported)
    MultipleLibraryTargets { target_count: usize },
}

impl fmt::Display for BundlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundlerError::Io { source, path } => {
                if let Some(path) = path {
                    write!(f, "IO error with file '{}': {source}", path.display())
                } else {
                    write!(f, "IO error: {source}")
                }
            }
            BundlerError::CargoMetadata { message, .. } => {
                write!(f, "Cargo metadata error: {message}")
            }
            BundlerError::Parsing { message, file_path } => {
                if let Some(path) = file_path {
                    write!(f, "Parsing error in '{}': {message}", path.display())
                } else {
                    write!(f, "Parsing error: {message}")
                }
            }
            BundlerError::ProjectStructure { message } => {
                write!(f, "Project structure error: {message}")
            }
            BundlerError::MultipleBinaryTargets { target_count } => {
                write!(
                    f,
                    "Multiple binary targets found ({target_count}). Only single binary target is supported."
                )
            }
            BundlerError::NoBinaryTarget => {
                write!(f, "No binary target found in the project")
            }
            BundlerError::MultipleLibraryTargets { target_count } => {
                write!(
                    f,
                    "Multiple library targets found ({target_count}). Only single library target is supported."
                )
            }
        }
    }
}

impl std::error::Error for BundlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BundlerError::Io { source, .. } => Some(source),
            BundlerError::CargoMetadata {
                source: Some(source),
                ..
            } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for BundlerError {
    fn from(error: io::Error) -> Self {
        BundlerError::Io {
            source: error,
            path: None,
        }
    }
}

impl From<cargo_metadata::Error> for BundlerError {
    fn from(error: cargo_metadata::Error) -> Self {
        BundlerError::CargoMetadata {
            message: error.to_string(),
            source: Some(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, BundlerError>;
