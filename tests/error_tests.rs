use cg_bundler::error::BundlerError;
use std::error::Error;
use std::io;
use std::path::PathBuf;

// ── Display ────────────────────────────────────────────────────────────

#[test]
fn test_display_io_with_path() {
    let err = BundlerError::Io {
        source: io::Error::new(io::ErrorKind::NotFound, "no file"),
        path: Some(PathBuf::from("/a/b.rs")),
    };
    let s = err.to_string();
    assert!(s.contains("IO error with file"));
    assert!(s.contains("/a/b.rs"));
    assert!(s.contains("no file"));
}

#[test]
fn test_display_io_without_path() {
    let err = BundlerError::Io {
        source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        path: None,
    };
    let s = err.to_string();
    assert!(s.starts_with("IO error:"));
    assert!(s.contains("denied"));
}

#[test]
fn test_display_cargo_metadata() {
    let err = BundlerError::CargoMetadata {
        message: "bad toml".into(),
        source: None,
    };
    let s = err.to_string();
    assert!(s.contains("Cargo metadata error"));
    assert!(s.contains("bad toml"));
}

#[test]
fn test_display_parsing_with_path() {
    let err = BundlerError::Parsing {
        message: "unexpected token".into(),
        file_path: Some(PathBuf::from("/src/main.rs")),
    };
    let s = err.to_string();
    assert!(s.contains("Parsing error in"));
    assert!(s.contains("main.rs"));
    assert!(s.contains("unexpected token"));
}

#[test]
fn test_display_parsing_without_path() {
    let err = BundlerError::Parsing {
        message: "bad".into(),
        file_path: None,
    };
    let s = err.to_string();
    assert!(s.starts_with("Parsing error:"));
    assert!(s.contains("bad"));
}

#[test]
fn test_display_project_structure() {
    let err = BundlerError::ProjectStructure {
        message: "missing src".into(),
    };
    assert!(err.to_string().contains("Project structure error"));
    assert!(err.to_string().contains("missing src"));
}

#[test]
fn test_display_multiple_binary_targets() {
    let err = BundlerError::MultipleBinaryTargets { target_count: 3 };
    let s = err.to_string();
    assert!(s.contains("Multiple binary targets found (3)"));
}

#[test]
fn test_display_no_binary_target() {
    assert!(
        BundlerError::NoBinaryTarget
            .to_string()
            .contains("No binary target")
    );
}

#[test]
fn test_display_multiple_library_targets() {
    let err = BundlerError::MultipleLibraryTargets { target_count: 2 };
    let s = err.to_string();
    assert!(s.contains("Multiple library targets found (2)"));
}

// ── Error::source ──────────────────────────────────────────────────────

#[test]
fn test_error_source_io() {
    let err = BundlerError::Io {
        source: io::Error::other("src"),
        path: None,
    };
    assert!(err.source().is_some());
}

#[test]
fn test_error_source_cargo_metadata_none() {
    let err = BundlerError::CargoMetadata {
        message: "m".into(),
        source: None,
    };
    assert!(err.source().is_none());
}

#[test]
fn test_error_source_other_variants() {
    assert!(BundlerError::NoBinaryTarget.source().is_none());
    assert!(
        BundlerError::ProjectStructure {
            message: "x".into()
        }
        .source()
        .is_none()
    );
    assert!(
        BundlerError::MultipleBinaryTargets { target_count: 1 }
            .source()
            .is_none()
    );
    assert!(
        BundlerError::MultipleLibraryTargets { target_count: 1 }
            .source()
            .is_none()
    );
    assert!(
        BundlerError::Parsing {
            message: "x".into(),
            file_path: None
        }
        .source()
        .is_none()
    );
}

// ── From impls ─────────────────────────────────────────────────────────

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "gone");
    let bundler_err: BundlerError = io_err.into();
    match bundler_err {
        BundlerError::Io { path, .. } => assert!(path.is_none()),
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_from_cargo_metadata_error_and_source() {
    // Run a cargo metadata command that will fail to obtain a real cargo_metadata::Error
    let meta_result = cargo_metadata::MetadataCommand::new()
        .manifest_path("/nonexistent_xyz/Cargo.toml")
        .exec();
    let Err(meta_err) = meta_result else {
        return; // Unlikely but skip if it somehow succeeds
    };
    let bundler_err: BundlerError = meta_err.into();
    match &bundler_err {
        BundlerError::CargoMetadata { message, source } => {
            assert!(!message.is_empty());
            assert!(source.is_some());
        }
        _ => panic!("Expected CargoMetadata variant"),
    }
    // Covers the error_source path for CargoMetadata with Some(source)
    assert!(bundler_err.source().is_some());
}
