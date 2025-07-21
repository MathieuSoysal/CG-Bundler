use std::path::Path;

use crate::cargo_project::CargoProject;
use crate::error::{BundlerError, Result};
use crate::file_manager::FileManager;
use crate::transformer::{CodeTransformer, TransformConfig};

/// Main bundler that orchestrates the bundling process
pub struct Bundler {
    config: TransformConfig,
}

impl Bundler {
    /// Create a new bundler with default configuration
    pub fn new() -> Self {
        Self {
            config: TransformConfig::default(),
        }
    }

    /// Create a new bundler with custom configuration
    pub fn with_config(config: TransformConfig) -> Self {
        Self { config }
    }

    /// Bundle a Cargo package into a single source file
    pub fn bundle<P: AsRef<Path>>(&self, package_path: P) -> Result<String> {
        let project = CargoProject::new(package_path)?;
        self.bundle_project(&project)
    }

    /// Bundle a CargoProject into a single source file
    pub fn bundle_project(&self, project: &CargoProject) -> Result<String> {
        let binary_source_path = project.binary_source_path();

        let code =
            FileManager::read_file(binary_source_path).map_err(|e| BundlerError::Parsing {
                message: format!("Failed to read binary target source: {e}"),
                file_path: Some(binary_source_path.to_path_buf()),
            })?;

        let mut file = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
            message: format!("Failed to parse binary target source: {e}"),
            file_path: Some(binary_source_path.to_path_buf()),
        })?;

        let mut transformer = CodeTransformer::new(
            project.base_path(),
            project.crate_name(),
            self.config.clone(),
        );

        transformer.transform_file(&mut file)?;

        let bundled_code = prettyplease::unparse(&file);
        Ok(bundled_code)
    }

    /// Get the current configuration
    pub fn config(&self) -> &TransformConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: TransformConfig) {
        self.config = config;
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new()
    }
}
