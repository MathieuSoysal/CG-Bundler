use std::collections::HashMap;
use std::path::{Path, PathBuf};

use syn::visit_mut::VisitMut;

use crate::error::{BundlerError, Result};

mod docs;
mod expansion;
mod macro_paths;
mod visit_mut_impl;

/// Configuration for code transformation
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct TransformConfig {
    pub remove_tests: bool,
    pub remove_docs: bool,
    pub expand_modules: bool,
    pub minify: bool,
    pub aggressive_minify: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            remove_tests: true,
            remove_docs: true,
            expand_modules: true,
            minify: false,
            aggressive_minify: false,
        }
    }
}

/// Handles code transformation and expansion
pub struct CodeTransformer<'a> {
    pub(super) base_path: &'a Path,
    pub(super) crate_name: &'a str,
    pub(super) config: TransformConfig,
    /// Mapping from external dependency crate names to their library entry-point paths.
    pub(super) external_libs: HashMap<String, PathBuf>,
    /// Explicit path of this crate's library root; defaults to `<base_path>/lib.rs`.
    pub(super) library_path: Option<&'a Path>,
    /// `false` for transformers operating inside a nested module of the bundle.
    pub(super) is_root: bool,
    /// Names declared by the module currently being rewritten. A dependency whose
    /// name appears here is shadowed and must not be retargeted at the bundle root.
    pub(super) shadowed_roots: Vec<String>,
    /// First error encountered while visiting the AST, surfaced by `transform_file`.
    pub(super) error: Option<BundlerError>,
}

impl<'a> CodeTransformer<'a> {
    /// Create a new code transformer with no external library knowledge.
    #[must_use]
    pub fn new(base_path: &'a Path, crate_name: &'a str, config: TransformConfig) -> Self {
        Self::with_external_libs(base_path, crate_name, config, HashMap::new())
    }

    /// Create a new code transformer that can also inline external crate dependencies.
    #[must_use]
    pub const fn with_external_libs(
        base_path: &'a Path,
        crate_name: &'a str,
        config: TransformConfig,
        external_libs: HashMap<String, PathBuf>,
    ) -> Self {
        Self {
            base_path,
            crate_name,
            config,
            external_libs,
            library_path: None,
            is_root: true,
            shadowed_roots: Vec::new(),
            error: None,
        }
    }

    /// Set the explicit path of this crate's library root.
    ///
    /// Required for packages declaring a custom `[lib] path`, where the library root
    /// is not `<base_path>/lib.rs`.
    #[must_use]
    pub const fn with_library_path(mut self, library_path: &'a Path) -> Self {
        self.library_path = Some(library_path);
        self
    }

    /// Build a transformer for a nested module, rooted at `base_path`.
    pub(super) fn child<'b>(&self, base_path: &'b Path) -> CodeTransformer<'b>
    where
        'a: 'b,
    {
        CodeTransformer {
            base_path,
            crate_name: self.crate_name,
            config: self.config.clone(),
            external_libs: self.external_libs.clone(),
            library_path: self.library_path,
            is_root: false,
            // Rust module scopes do not nest for path roots, so a child starts empty.
            shadowed_roots: Vec::new(),
            error: None,
        }
    }

    /// Record the first error seen while visiting; `VisitMut` cannot return one.
    pub(super) fn record_error(&mut self, error: BundlerError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    /// Resolve this crate's library root file.
    pub(super) fn library_source_path(&self) -> PathBuf {
        self.library_path
            .map_or_else(|| self.base_path.join("lib.rs"), Path::to_path_buf)
    }

    /// Transform a file's AST according to configuration.
    ///
    /// # Errors
    /// Returns an error if module expansion fails.
    pub fn transform_file(&mut self, file: &mut syn::File) -> Result<()> {
        if self.config.remove_docs {
            self.remove_file_level_docs(file);
        }

        self.expand_items(&mut file.items)?;

        for item in &mut file.items {
            self.visit_item_mut(item);
        }

        if let Some(error) = self.error.take() {
            return Err(error);
        }

        // External crates are inlined last so that dependencies referenced only from
        // nested modules are still detected.
        if self.is_root {
            self.expand_external_libs(&mut file.items)?;
        }

        Ok(())
    }

    /// Expand items (extern crate, use paths, etc.).
    ///
    /// # Errors
    /// Returns an error if module expansion or file parsing fails.
    pub fn expand_items(&mut self, items: &mut Vec<syn::Item>) -> Result<()> {
        if self.config.expand_modules {
            if self.is_root {
                let has_extern_crate = items
                    .iter()
                    .any(|item| Self::is_extern_crate(item, self.crate_name));
                let has_use_statement = items
                    .iter()
                    .any(|item| Self::is_use_path(item, self.crate_name));

                if has_extern_crate {
                    self.expand_extern_crate(items)?;
                    self.remove_use_paths(items);
                } else if has_use_statement {
                    self.expand_use_path(items)?;
                }
            } else {
                // The library and every dependency are inlined at the bundle root, so
                // imports inside a module must be rewritten to go through `crate::`.
                self.retarget_imports(items);
            }
        }

        if self.config.remove_tests || self.config.remove_docs {
            self.filter_tests_and_docs(items);
        }

        Ok(())
    }
}
