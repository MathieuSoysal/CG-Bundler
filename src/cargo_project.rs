use cargo_metadata::{Metadata, Package, Target};
use std::path::{Path, PathBuf};

use crate::error::{BundlerError, Result};

/// Represents a Cargo project with its metadata
#[derive(Debug, Clone)]
pub struct CargoProject {
    metadata: Metadata,
    root_package: Package,
    binary_target: Target,
    library_target: Option<Target>,
    base_path: PathBuf,
}

impl CargoProject {
    /// Create a new CargoProject by analyzing the given path
    pub fn new<P: AsRef<Path>>(package_path: P) -> Result<Self> {
        let package_path = package_path.as_ref();
        let manifest_path = package_path.join("Cargo.toml");

        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()
            .map_err(|e| BundlerError::CargoMetadata {
                message: format!("Failed to obtain cargo metadata: {e}"),
                source: Some(e),
            })?;

        let root_package = Self::find_root_package(&metadata, &manifest_path)?;
        let (binary_target, library_target) = Self::analyze_targets(&root_package)?;
        let base_path = Self::determine_base_path(&library_target, &binary_target)?;

        Ok(Self {
            metadata,
            root_package,
            binary_target,
            library_target,
            base_path,
        })
    }

    /// Get the root package
    pub fn root_package(&self) -> &Package {
        &self.root_package
    }

    /// Get the binary target
    pub fn binary_target(&self) -> &Target {
        &self.binary_target
    }

    /// Get the library target if it exists
    pub fn library_target(&self) -> Option<&Target> {
        self.library_target.as_ref()
    }

    /// Get the base path for source files
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get the crate name (from library or binary target)
    pub fn crate_name(&self) -> &str {
        self.library_target
            .as_ref()
            .map(|lib| lib.name.as_str())
            .unwrap_or(&self.binary_target.name)
    }

    /// Get the binary source path
    pub fn binary_source_path(&self) -> &Path {
        Path::new(&self.binary_target.src_path)
    }

    /// Get the library source path if it exists
    pub fn library_source_path(&self) -> Option<&Path> {
        self.library_target
            .as_ref()
            .map(|lib| Path::new(&lib.src_path))
    }

    /// Find the root package in the metadata
    fn find_root_package(metadata: &Metadata, manifest_path: &Path) -> Result<Package> {
        let canonical_manifest =
            std::fs::canonicalize(manifest_path).unwrap_or_else(|_| manifest_path.to_path_buf());

        metadata
            .packages
            .iter()
            .find(|pkg| {
                let pkg_canonical = std::fs::canonicalize(&pkg.manifest_path)
                    .unwrap_or_else(|_| pkg.manifest_path.clone().into());
                pkg_canonical == canonical_manifest
            })
            .cloned()
            .ok_or_else(|| BundlerError::ProjectStructure {
                message: "Failed to find root package in metadata".to_string(),
            })
    }

    /// Analyze targets and extract binary and library targets
    fn analyze_targets(package: &Package) -> Result<(Target, Option<Target>)> {
        let targets = &package.targets;

        // Find binary targets
        let binary_targets: Vec<_> = targets
            .iter()
            .filter(|t| Self::target_is(t, "bin"))
            .collect();

        if binary_targets.is_empty() {
            return Err(BundlerError::NoBinaryTarget);
        }

        if binary_targets.len() > 1 {
            return Err(BundlerError::MultipleBinaryTargets {
                target_count: binary_targets.len(),
            });
        }

        let binary_target = binary_targets[0].clone();

        // Find library targets
        let library_targets: Vec<_> = targets
            .iter()
            .filter(|t| Self::target_is(t, "lib"))
            .collect();

        if library_targets.len() > 1 {
            return Err(BundlerError::MultipleLibraryTargets {
                target_count: library_targets.len(),
            });
        }

        let library_target = library_targets.first().map(|t| (*t).clone());

        Ok((binary_target, library_target))
    }

    /// Determine the base path for source files
    fn determine_base_path(
        library_target: &Option<Target>,
        binary_target: &Target,
    ) -> Result<PathBuf> {
        let reference_target = library_target.as_ref().unwrap_or(binary_target);

        Path::new(&reference_target.src_path)
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| BundlerError::ProjectStructure {
                message: "Source path has no parent directory".to_string(),
            })
    }

    /// Check if a target has a specific kind
    fn target_is(target: &Target, target_kind: &str) -> bool {
        target.kind.iter().any(|kind| kind == target_kind)
    }

    /// Get the cargo metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(temp_dir: &Path, has_lib: bool) -> PathBuf {
        let project_path = temp_dir.join("test_project");
        fs::create_dir_all(&project_path).unwrap();

        let mut cargo_toml = r#"
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test_project"
path = "src/main.rs"
"#
        .to_string();

        if has_lib {
            cargo_toml.push_str(
                r#"
[lib]
name = "test_project"
path = "src/lib.rs"
"#,
            );
        }

        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let src_dir = project_path.join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        if has_lib {
            fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();
        }

        project_path
    }

    #[test]
    fn test_project_with_lib() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project(temp_dir.path(), true);

        let project = CargoProject::new(&project_path).unwrap();

        assert_eq!(project.crate_name(), "test_project");
        assert!(project.library_target().is_some());
        assert_eq!(project.binary_target().name, "test_project");
    }

    #[test]
    fn test_project_without_lib() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project(temp_dir.path(), false);

        let project = CargoProject::new(&project_path).unwrap();

        assert_eq!(project.crate_name(), "test_project");
        assert!(project.library_target().is_none());
        assert_eq!(project.binary_target().name, "test_project");
    }

    #[test]
    fn test_nonexistent_project() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let result = CargoProject::new(&nonexistent_path);
        assert!(result.is_err());
    }
}
