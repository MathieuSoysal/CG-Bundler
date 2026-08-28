use cargo_metadata::{Metadata, Package, Target};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{BundlerError, Result};

/// Represents a Cargo project with its metadata
#[derive(Debug, Clone)]
pub struct CargoProject {
    metadata: Metadata,
    root_package: Package,
    binary_target: Target,
    library_target: Option<Target>,
    normalized_crate_name: String,
    base_path: PathBuf,
}

impl CargoProject {
    /// Create a new `CargoProject` by analyzing the given path
    ///
    /// # Errors
    /// Returns an error if the Cargo project cannot be analyzed or parsed
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
        let normalized_crate_name =
            Self::determine_crate_name(library_target.as_ref(), &binary_target);
        let base_path = Self::determine_base_path(library_target.as_ref(), &binary_target)?;

        Ok(Self {
            metadata,
            root_package,
            binary_target,
            library_target,
            normalized_crate_name,
            base_path,
        })
    }

    /// Get the root package
    #[must_use]
    pub const fn root_package(&self) -> &Package {
        &self.root_package
    }

    /// Get the binary target
    #[must_use]
    pub const fn binary_target(&self) -> &Target {
        &self.binary_target
    }

    /// Get the library target if it exists
    #[must_use]
    pub const fn library_target(&self) -> Option<&Target> {
        self.library_target.as_ref()
    }

    /// Get the base path for source files
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get the crate name (prefer library name over binary name)
    #[must_use]
    pub fn crate_name(&self) -> &str {
        &self.normalized_crate_name
    }

    /// Get the binary source path
    #[must_use]
    pub fn binary_source_path(&self) -> &Path {
        Path::new(&self.binary_target.src_path)
    }

    /// Get the library source path if it exists
    #[must_use]
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
        library_target: Option<&Target>,
        binary_target: &Target,
    ) -> Result<PathBuf> {
        let reference_target = library_target.unwrap_or(binary_target);

        Path::new(&reference_target.src_path)
            .parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| BundlerError::ProjectStructure {
                message: "Source path has no parent directory".to_string(),
            })
    }

    /// Determine crate name used in Rust paths.
    fn determine_crate_name(library_target: Option<&Target>, binary_target: &Target) -> String {
        let raw_name = library_target.map_or(binary_target.name.as_str(), |lib| lib.name.as_str());
        Self::normalize_crate_identifier(raw_name)
    }

    /// Normalize Cargo target names to valid Rust crate identifiers.
    fn normalize_crate_identifier(name: &str) -> String {
        name.replace('-', "_")
    }

    /// Check if a target has a specific kind
    fn target_is(target: &Target, target_kind: &str) -> bool {
        use cargo_metadata::TargetKind;
        target.kind.iter().any(|kind| match kind {
            TargetKind::Bin if target_kind == "bin" => true,
            TargetKind::Lib if target_kind == "lib" => true,
            _ => false,
        })
    }

    /// Get the cargo metadata
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Return a map from each direct dependency's Rust crate name to the `PathBuf`
    /// of its library entry point (`lib.rs` or equivalent).
    ///
    /// Only *path* dependencies that expose a library target are included. Registry
    /// dependencies are excluded: their sources rely on build scripts, `#[path]`
    /// modules and feature-gated files that cannot be inlined into a single file.
    /// The root package itself is excluded.
    #[must_use]
    pub fn external_lib_paths(&self) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();

        let Some(root_node) = self.root_resolve_node() else {
            return map;
        };

        for dep in &self.root_package.dependencies {
            let crate_name = Self::dependency_crate_name(dep);

            for pkg in self.dependency_packages(root_node, &crate_name) {
                if pkg.source.is_some() {
                    continue;
                }

                if let Some(lib_target) = pkg.targets.iter().find(|t| Self::target_is(t, "lib")) {
                    map.insert(
                        crate_name.clone(),
                        lib_target.src_path.clone().into_std_path_buf(),
                    );
                    break;
                }
            }
        }

        map
    }

    fn dependency_crate_name(dep: &cargo_metadata::Dependency) -> String {
        dep.rename.as_deref().map_or_else(
            || Self::normalize_crate_identifier(&dep.name),
            Self::normalize_crate_identifier,
        )
    }

    fn root_resolve_node(&self) -> Option<&cargo_metadata::Node> {
        let resolve = self.metadata.resolve.as_ref()?;
        resolve
            .nodes
            .iter()
            .find(|node| node.id == self.root_package.id)
    }

    fn dependency_packages<'a>(
        &'a self,
        root_node: &'a cargo_metadata::Node,
        crate_name: &str,
    ) -> impl Iterator<Item = &'a Package> {
        root_node
            .deps
            .iter()
            .filter(move |node_dep| Self::normalize_crate_identifier(&node_dep.name) == crate_name)
            .filter_map(|node_dep| self.package_by_id(&node_dep.pkg))
            .filter(move |pkg| pkg.id != self.root_package.id)
    }

    fn package_by_id(&self, id: &cargo_metadata::PackageId) -> Option<&Package> {
        self.metadata.packages.iter().find(|p| p.id == *id)
    }
}
