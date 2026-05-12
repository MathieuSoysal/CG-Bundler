use std::mem;
use std::path::{Path, PathBuf};

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

use crate::error::{BundlerError, Result};
use crate::file_manager::FileManager;

use super::CodeTransformer;

struct PathRootVisitor<'a> {
    crate_name: &'a str,
    found: bool,
}

impl<'ast> Visit<'ast> for PathRootVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(first) = path.segments.first()
            && first.ident == self.crate_name
        {
            self.found = true;
            return;
        }

        syn::visit::visit_path(self, path);
    }
}

impl CodeTransformer<'_> {
    /// Wrap each referenced external dependency's source into a `mod <name> { ... }` block
    /// inserted at the head of `items`, leaving the original `use` statements in place so
    /// they now resolve against the newly created module.
    pub(super) fn expand_external_libs(&self, items: &mut Vec<syn::Item>) -> Result<()> {
        let mut to_expand: Vec<(String, PathBuf)> = self
            .external_libs
            .iter()
            .filter(|(name, _)| {
                items.iter().any(|item| {
                    Self::is_use_path(item, name) || Self::item_references_crate(item, name)
                })
            })
            .map(|(name, path)| (name.clone(), path.clone()))
            .collect();

        to_expand.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        for (ext_name, ext_lib_path) in to_expand {
            self.expand_external_lib(items, &ext_name, &ext_lib_path)?;
        }

        Ok(())
    }

    /// Inline a single external crate's library as a `mod <name> { ... }` block.
    fn expand_external_lib(
        &self,
        items: &mut Vec<syn::Item>,
        ext_name: &str,
        ext_lib_path: &Path,
    ) -> Result<()> {
        if Self::has_module_named(items, ext_name) {
            return Ok(());
        }

        let code =
            FileManager::read_file(ext_lib_path).map_err(|e| BundlerError::ProjectStructure {
                message: format!(
                    "Failed to read external crate '{ext_name}' from '{}': {e}",
                    ext_lib_path.display()
                ),
            })?;

        let mut lib_file = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
            message: format!("Failed to parse external crate '{ext_name}': {e}"),
            file_path: Some(ext_lib_path.to_path_buf()),
        })?;

        let ext_base_path =
            ext_lib_path
                .parent()
                .ok_or_else(|| BundlerError::ProjectStructure {
                    message: format!(
                        "External crate '{ext_name}' lib path has no parent directory"
                    ),
                })?;

        let mut expander = CodeTransformer::new(ext_base_path, ext_name, self.config.clone());
        expander.expand_items(&mut lib_file.items)?;
        for item in &mut lib_file.items {
            expander.visit_item_mut(item);
        }

        let mod_item = syn::Item::Mod(syn::ItemMod {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            unsafety: None,
            mod_token: syn::token::Mod::default(),
            ident: syn::Ident::new(ext_name, proc_macro2::Span::call_site()),
            content: Some((syn::token::Brace::default(), lib_file.items)),
            semi: None,
        });

        items.insert(0, mod_item);

        Ok(())
    }

    fn has_module_named(items: &[syn::Item], module_name: &str) -> bool {
        items.iter().any(|item| {
            if let syn::Item::Mod(item_mod) = item {
                return item_mod.ident == module_name;
            }
            false
        })
    }

    fn item_references_crate(item: &syn::Item, crate_name: &str) -> bool {
        let mut visitor = PathRootVisitor {
            crate_name,
            found: false,
        };
        visitor.visit_item(item);
        visitor.found
    }

    /// Expand extern crate declarations.
    pub(super) fn expand_extern_crate(&self, items: &mut Vec<syn::Item>) -> Result<()> {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if Self::is_extern_crate(&item, self.crate_name) {
                eprintln!(
                    "Expanding crate {} in {}",
                    self.crate_name,
                    self.base_path.display()
                );
                let lib_path = self.base_path.join("lib.rs");
                let code = FileManager::read_file(&lib_path).map_err(|_| {
                    BundlerError::ProjectStructure {
                        message: "Failed to read lib.rs for extern crate expansion".to_string(),
                    }
                })?;

                let lib = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
                    message: format!("Failed to parse lib.rs: {e}"),
                    file_path: Some(lib_path),
                })?;

                new_items.extend(lib.items);
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
        Ok(())
    }

    /// Remove use paths without expanding library (used when extern crate is present).
    pub(super) fn remove_use_paths(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if !Self::is_use_path(&item, self.crate_name) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    /// Expand use paths.
    pub(super) fn expand_use_path(&self, items: &mut Vec<syn::Item>) -> Result<()> {
        let mut new_items = vec![];
        let mut library_expanded = false;

        for item in items.drain(..) {
            if Self::is_use_path(&item, self.crate_name) {
                if !library_expanded {
                    eprintln!(
                        "Expanding crate {} in {} (from use statement)",
                        self.crate_name,
                        self.base_path.display()
                    );
                    let lib_path = self.base_path.join("lib.rs");
                    let code = FileManager::read_file(&lib_path).map_err(|_| {
                        BundlerError::ProjectStructure {
                            message: "Failed to read lib.rs for use path expansion".to_string(),
                        }
                    })?;

                    let lib = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
                        message: format!("Failed to parse lib.rs: {e}"),
                        file_path: Some(lib_path),
                    })?;

                    new_items.extend(lib.items);
                    library_expanded = true;
                }
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
        Ok(())
    }

    /// Expand module declarations.
    pub(super) fn expand_mods(&self, item: &mut syn::ItemMod) -> Result<()> {
        if item.content.is_some() {
            return Ok(());
        }

        let name = item.ident.to_string();
        let (base_path, code) = FileManager::find_module_file(self.base_path, &name)?;

        let mut file = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
            message: format!("Failed to parse module file: {e}"),
            file_path: Some(base_path.join(format!("{name}.rs"))),
        })?;

        let mut expander = CodeTransformer::new(&base_path, self.crate_name, self.config.clone());
        expander.expand_items(&mut file.items)?;

        for item in &mut file.items {
            expander.visit_item_mut(item);
        }

        item.content = Some((syn::token::Brace::default(), file.items));
        Ok(())
    }

    /// Expand crate paths.
    pub fn expand_crate_path(&self, path: &mut syn::Path) {
        if path.segments.len() > 1 && Self::path_starts_with(path, self.crate_name) {
            let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                .into_pairs()
                .skip(1)
                .collect();
            path.segments = new_segments;
        }
    }

    /// Check if item is an extern crate declaration.
    #[must_use]
    pub fn is_extern_crate(item: &syn::Item, crate_name: &str) -> bool {
        if let syn::Item::ExternCrate(ref item) = *item
            && item.ident == crate_name
        {
            return true;
        }
        false
    }

    /// Check if path starts with a specific segment.
    fn path_starts_with(path: &syn::Path, segment: &str) -> bool {
        if let Some(el) = path.segments.first()
            && el.ident == segment
        {
            return true;
        }
        false
    }

    /// Check if item is a use path that references the crate.
    #[must_use]
    pub fn is_use_path(item: &syn::Item, first_segment: &str) -> bool {
        if let syn::Item::Use(ref item) = *item {
            return Self::use_tree_references_crate(&item.tree, first_segment);
        }
        false
    }

    /// Check if a use tree references the specified crate.
    #[must_use]
    pub fn use_tree_references_crate(tree: &syn::UseTree, crate_name: &str) -> bool {
        match tree {
            syn::UseTree::Path(path) => path.ident == crate_name,
            syn::UseTree::Name(name) => name.ident == crate_name,
            syn::UseTree::Rename(rename) => rename.ident == crate_name,
            syn::UseTree::Glob(_) | syn::UseTree::Group(_) => false,
        }
    }
}
