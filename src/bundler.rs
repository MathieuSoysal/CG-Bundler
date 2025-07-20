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
        
        let code = FileManager::read_file(binary_source_path)
            .map_err(|e| BundlerError::Parsing {
                message: format!("Failed to read binary target source: {}", e),
                file_path: Some(binary_source_path.to_path_buf()),
            })?;

        let mut file = syn::parse_file(&code)
            .map_err(|e| BundlerError::Parsing {
                message: format!("Failed to parse binary target source: {}", e),
                file_path: Some(binary_source_path.to_path_buf()),
            })?;

        let mut transformer = CodeTransformer::new(
            project.base_path(),
            project.crate_name(),
            self.config.clone(),
        );

        transformer.transform_file(&mut file)?;

        Ok(quote::quote!(#file).to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(temp_dir: &Path, name: &str, has_lib: bool) -> std::path::PathBuf {
        let project_path = temp_dir.join(name);
        fs::create_dir_all(&project_path).unwrap();
        
        let mut cargo_toml = format!(r#"
[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#);

        if has_lib {
            cargo_toml.push_str(&format!(r#"
[lib]
name = "{name}"
path = "src/lib.rs"
"#));
        }
        
        fs::write(project_path.join("Cargo.toml"), cargo_toml).unwrap();
        
        let src_dir = project_path.join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let main_content = if has_lib {
            format!("extern crate {name};\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}", name)
        } else {
            "fn main() {\n    println!(\"Hello, World!\");\n}".to_string()
        };
        
        fs::write(src_dir.join("main.rs"), main_content).unwrap();
        
        if has_lib {
            fs::write(src_dir.join("lib.rs"), "pub fn hello() -> &'static str {\n    \"Hello from lib!\"\n}").unwrap();
        }
        
        project_path
    }

    #[test]
    fn test_bundle_project_with_lib() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project(temp_dir.path(), "test_with_lib", true);
        
        let bundler = Bundler::new();
        let result = bundler.bundle(&project_path).unwrap();
        
        assert!(!result.is_empty());
        assert!(result.contains("fn main"));
        assert!(result.contains("pub fn hello"));
    }

    #[test]
    fn test_bundle_project_without_lib() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project(temp_dir.path(), "test_without_lib", false);
        
        let bundler = Bundler::new();
        let result = bundler.bundle(&project_path).unwrap();
        
        assert!(!result.is_empty());
        assert!(result.contains("fn main"));
        assert!(result.contains("Hello, World!"));
    }

    #[test]
    fn test_bundle_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_project(temp_dir.path(), "test_custom_config", false);
        
        let config = TransformConfig {
            remove_tests: false,
            remove_docs: false,
            expand_modules: false,
        };
        
        let bundler = Bundler::with_config(config);
        let result = bundler.bundle(&project_path).unwrap();
        
        assert!(!result.is_empty());
        assert!(result.contains("fn main"));
    }

    #[test]
    fn test_bundle_nonexistent_project() {
        let bundler = Bundler::new();
        let result = bundler.bundle("/nonexistent/project");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_default_bundler() {
        let bundler = Bundler::default();
        assert!(bundler.config().remove_tests);
        assert!(bundler.config().remove_docs);
        assert!(bundler.config().expand_modules);
    }

    #[test]
    fn test_set_config() {
        let mut bundler = Bundler::new();
        
        let new_config = TransformConfig {
            remove_tests: false,
            remove_docs: true,
            expand_modules: false,
        };
        
        bundler.set_config(new_config.clone());
        
        assert_eq!(bundler.config().remove_tests, new_config.remove_tests);
        assert_eq!(bundler.config().remove_docs, new_config.remove_docs);
        assert_eq!(bundler.config().expand_modules, new_config.expand_modules);
    }
}
