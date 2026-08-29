use std::mem;
use std::path::{Path, PathBuf};

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

use crate::error::{BundlerError, Result};
use crate::file_manager::FileManager;

use super::CodeTransformer;
use super::macro_paths::{RootRewrite, rewrite_root_in_tokens, tokens_reference_root};

struct PathRootVisitor<'a> {
    crate_name: &'a str,
    found: bool,
}

impl PathRootVisitor<'_> {
    /// Matches both `<crate>::..` and the already-requalified `crate::<crate>::..`.
    fn matches(&self, path: &syn::Path) -> bool {
        let mut segments = path.segments.iter();
        let Some(first) = segments.next() else {
            return false;
        };

        if first.ident == self.crate_name {
            return true;
        }

        first.ident == "crate"
            && segments
                .next()
                .is_some_and(|second| second.ident == self.crate_name)
    }
}

impl<'ast> Visit<'ast> for PathRootVisitor<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if self.matches(path) {
            self.found = true;
            return;
        }

        syn::visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if tokens_reference_root(&mac.tokens, self.crate_name) {
            self.found = true;
            return;
        }

        syn::visit::visit_macro(self, mac);
    }
}

/// Rewrites `crate::..` to `crate::<module>::..` for code being moved into `mod <module>`.
struct CrateRootRequalifier<'a> {
    module: &'a str,
}

impl CrateRootRequalifier<'_> {
    fn requalify(&self, path: &mut syn::Path) {
        let Some(first) = path.segments.first() else {
            return;
        };
        if first.ident != "crate" {
            return;
        }

        let span = first.ident.span();
        let tail = mem::replace(&mut path.segments, Punctuated::new());
        path.segments
            .push(syn::PathSegment::from(syn::Ident::new("crate", span)));
        path.segments
            .push(syn::PathSegment::from(syn::Ident::new(self.module, span)));
        path.segments.extend(tail.into_pairs().skip(1));
    }
}

impl VisitMut for CrateRootRequalifier<'_> {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.requalify(path);
        for mut pair in Punctuated::pairs_mut(&mut path.segments) {
            self.visit_path_segment_mut(pair.value_mut());
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        self.visit_path_mut(&mut mac.path);

        let tokens = mem::take(&mut mac.tokens);
        mac.tokens = rewrite_root_in_tokens(tokens, "crate", &RootRewrite::Qualify(self.module));
    }

    fn visit_use_tree_mut(&mut self, tree: &mut syn::UseTree) {
        if let syn::UseTree::Path(path) = tree
            && path.ident == "crate"
        {
            let span = path.ident.span();
            let rest = mem::replace(
                path.tree.as_mut(),
                syn::UseTree::Glob(syn::UseGlob {
                    star_token: syn::token::Star::default(),
                }),
            );
            *path.tree = syn::UseTree::Path(syn::UsePath {
                ident: syn::Ident::new(self.module, span),
                colon2_token: syn::token::PathSep::default(),
                tree: Box::new(rest),
            });
            return;
        }

        syn::visit_mut::visit_use_tree_mut(self, tree);
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
            .filter(|(name, _)| Self::items_reference_crate(items, name))
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

        let mut expander = CodeTransformer::new(ext_base_path, ext_name, self.config.clone())
            .with_library_path(ext_lib_path);
        expander.transform_file(&mut lib_file)?;

        // The dependency now lives under `mod <ext_name>`, so its own `crate::` paths
        // must be requalified to reach it.
        let mut requalifier = CrateRootRequalifier { module: ext_name };
        for item in &mut lib_file.items {
            requalifier.visit_item_mut(item);
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

    /// Whether any item — including items nested in inline modules — refers to `crate_name`.
    fn items_reference_crate(items: &[syn::Item], crate_name: &str) -> bool {
        items.iter().any(|item| {
            if let syn::Item::Use(item_use) = item
                && Self::use_tree_targets_crate(&item_use.tree, crate_name)
            {
                return true;
            }

            if Self::item_references_crate(item, crate_name) {
                return true;
            }

            if let syn::Item::Mod(item_mod) = item
                && let Some((_, inner)) = item_mod.content.as_ref()
            {
                return Self::items_reference_crate(inner, crate_name);
            }

            false
        })
    }

    /// Matches `use <crate>::..` as well as the requalified `use crate::<crate>::..`.
    fn use_tree_targets_crate(tree: &syn::UseTree, crate_name: &str) -> bool {
        if Self::use_tree_references_crate(tree, crate_name) {
            return true;
        }

        match tree {
            syn::UseTree::Path(path) if path.ident == "crate" => {
                Self::use_tree_references_crate(&path.tree, crate_name)
            }
            _ => false,
        }
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
        let lib_items = self.read_library_items()?;
        let mut new_items = vec![];
        let mut library_expanded = false;

        for item in items.drain(..) {
            if Self::is_extern_crate(&item, self.crate_name) {
                if !library_expanded {
                    new_items.extend(lib_items.iter().cloned());
                    library_expanded = true;
                }
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
        Ok(())
    }

    /// Remove use paths without expanding library (used when extern crate is present).
    pub(super) fn remove_use_paths(&self, items: &mut Vec<syn::Item>) {
        items.retain(|item| !Self::is_use_path(item, self.crate_name));
    }

    /// Expand use paths.
    pub(super) fn expand_use_path(&self, items: &mut Vec<syn::Item>) -> Result<()> {
        let lib_items = self.read_library_items()?;
        let mut new_items = vec![];
        let mut library_expanded = false;

        for item in items.drain(..) {
            if Self::is_use_path(&item, self.crate_name) {
                if !library_expanded {
                    new_items.extend(lib_items.iter().cloned());
                    library_expanded = true;
                }
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
        Ok(())
    }

    /// Read and parse this crate's library root.
    fn read_library_items(&self) -> Result<Vec<syn::Item>> {
        let lib_path = self.library_source_path();

        let code =
            FileManager::read_file(&lib_path).map_err(|e| BundlerError::ProjectStructure {
                message: format!(
                    "Failed to read library root '{}' for crate expansion: {e}",
                    lib_path.display()
                ),
            })?;

        let lib = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
            message: format!("Failed to parse library root: {e}"),
            file_path: Some(lib_path),
        })?;

        Ok(lib.items)
    }

    /// Rewrite imports inside a nested module so they resolve against the bundle root.
    ///
    /// `use <this_crate>::X` becomes `use crate::X`, and `use <dependency>::X` becomes
    /// `use crate::<dependency>::X`, matching where the inlined code actually lives.
    pub(super) fn retarget_imports(&self, items: &mut [syn::Item]) {
        for item in items {
            let syn::Item::Use(item_use) = item else {
                continue;
            };
            let syn::UseTree::Path(path) = &mut item_use.tree else {
                continue;
            };

            if path.ident == self.crate_name {
                item_use.leading_colon = None;
                path.ident = syn::Ident::new("crate", path.ident.span());
            } else if self.external_libs.contains_key(&path.ident.to_string()) {
                let span = path.ident.span();
                item_use.leading_colon = None;
                let inner = mem::replace(
                    &mut item_use.tree,
                    syn::UseTree::Glob(syn::UseGlob {
                        star_token: syn::token::Star::default(),
                    }),
                );
                item_use.tree = syn::UseTree::Path(syn::UsePath {
                    ident: syn::Ident::new("crate", span),
                    colon2_token: syn::token::PathSep::default(),
                    tree: Box::new(inner),
                });
            }
        }
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

        let mut expander = self.child(&base_path);
        expander.expand_items(&mut file.items)?;

        for item in &mut file.items {
            expander.visit_item_mut(item);
        }

        if let Some(error) = expander.error.take() {
            return Err(error);
        }

        item.content = Some((syn::token::Brace::default(), file.items));
        Ok(())
    }

    /// Expand crate paths.
    ///
    /// A leading `::` anchors the path at the extern prelude, which is where both
    /// this crate and its dependencies used to live. Bundling relocates them under
    /// the bundle root, so the anchor is dropped along with the rewrite -- leaving
    /// it in place would produce the un-parseable `::crate::..`.
    pub fn expand_crate_path(&self, path: &mut syn::Path) {
        if path.segments.len() < 2 {
            return;
        }

        if Self::path_starts_with(path, self.crate_name) {
            path.leading_colon = None;
            if self.is_root {
                let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                    .into_pairs()
                    .skip(1)
                    .collect();
                path.segments = new_segments;
            } else if let Some(first) = path.segments.first_mut() {
                // Library items sit at the bundle root, unreachable by bare name here.
                first.ident = syn::Ident::new("crate", first.ident.span());
                first.arguments = syn::PathArguments::None;
            }
            return;
        }

        // Dependencies are inlined as `mod <name>` at the bundle root.
        if !self.is_root
            && let Some(first) = path.segments.first()
            && self.external_libs.contains_key(&first.ident.to_string())
        {
            let span = first.ident.span();
            path.leading_colon = None;
            let tail = mem::replace(&mut path.segments, Punctuated::new());
            path.segments
                .push(syn::PathSegment::from(syn::Ident::new("crate", span)));
            path.segments.extend(tail.into_pairs());
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
