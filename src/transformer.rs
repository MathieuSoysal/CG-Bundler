use std::collections::HashMap;
use std::path::{Path, PathBuf};

use syn::visit_mut::VisitMut;

use crate::error::Result;

mod docs;
mod expansion;
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
}

impl<'a> CodeTransformer<'a> {
    /// Create a new code transformer with no external library knowledge.
    #[must_use]
    pub fn new(base_path: &'a Path, crate_name: &'a str, config: TransformConfig) -> Self {
        Self {
            base_path,
            crate_name,
            config,
            external_libs: HashMap::new(),
        }
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
        }
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

        Ok(())
    }

    /// Expand items (extern crate, use paths, etc.).
    ///
    /// # Errors
    /// Returns an error if module expansion or file parsing fails.
    pub fn expand_items(&mut self, items: &mut Vec<syn::Item>) -> Result<()> {
        if self.config.expand_modules {
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
        }

        self.expand_external_libs(items)?;

        if self.config.remove_tests || self.config.remove_docs {
            self.filter_tests_and_docs(items);
        }

        Ok(())
    }
}
