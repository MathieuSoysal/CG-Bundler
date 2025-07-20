use std::mem;
use std::path::Path;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

use crate::error::{BundlerError, Result};
use crate::file_manager::FileManager;

/// Configuration for code transformation
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub remove_tests: bool,
    pub remove_docs: bool,
    pub expand_modules: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            remove_tests: true,
            remove_docs: true,
            expand_modules: true,
        }
    }
}

/// Handles code transformation and expansion
pub struct CodeTransformer<'a> {
    base_path: &'a Path,
    crate_name: &'a str,
    config: TransformConfig,
}

impl<'a> CodeTransformer<'a> {
    /// Create a new code transformer
    pub fn new(base_path: &'a Path, crate_name: &'a str, config: TransformConfig) -> Self {
        Self {
            base_path,
            crate_name,
            config,
        }
    }

    /// Transform a file's AST according to configuration
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

    /// Expand items (extern crate, use paths, etc.)
    pub fn expand_items(&mut self, items: &mut Vec<syn::Item>) -> Result<()> {
        if self.config.expand_modules {
            self.expand_extern_crate(items)?;
            self.expand_use_path(items);
        }

        if self.config.remove_tests || self.config.remove_docs {
            self.filter_tests_and_docs(items);
        }

        Ok(())
    }

    /// Remove file-level documentation
    fn remove_file_level_docs(&self, file: &mut syn::File) {
        if self.config.remove_docs {
            file.attrs.retain(|attr| !self.is_doc_attribute(attr));
        }
    }

    /// Filter out tests and documentation
    fn filter_tests_and_docs(&self, items: &mut Vec<syn::Item>) {
        items.retain(|item| {
            if self.config.remove_tests && self.has_test_attribute(item) {
                return false;
            }
            true
        });

        if self.config.remove_docs {
            for item in items.iter_mut() {
                self.remove_doc_attributes(item);
                self.remove_doc_from_children(item);
            }
        }
    }

    /// Check if an attribute is a documentation attribute
    fn is_doc_attribute(&self, attr: &syn::Attribute) -> bool {
        if attr.path().is_ident("doc") {
            return true;
        }

        let attr_str = quote::quote!(#attr).to_string();
        attr_str.starts_with("# [doc")
            || attr_str.starts_with("#[doc")
            || attr_str.contains("doc =")
    }

    /// Remove documentation from child elements
    fn remove_doc_from_children(&self, item: &mut syn::Item) {
        match item {
            syn::Item::Struct(item_struct) => {
                self.remove_docs_from_fields(&mut item_struct.fields);
            }
            syn::Item::Enum(item_enum) => {
                for variant in &mut item_enum.variants {
                    variant.attrs.retain(|attr| !self.is_doc_attribute(attr));
                    self.remove_docs_from_fields(&mut variant.fields);
                }
            }
            syn::Item::Fn(item_fn) => {
                self.remove_docs_from_fn_inputs(&mut item_fn.sig.inputs);
            }
            syn::Item::Impl(item_impl) => {
                for impl_item in &mut item_impl.items {
                    self.remove_docs_from_impl_item(impl_item);
                }
            }
            syn::Item::Trait(item_trait) => {
                for trait_item in &mut item_trait.items {
                    self.remove_docs_from_trait_item(trait_item);
                }
            }
            syn::Item::Mod(item_mod) => {
                if let Some((_, ref mut mod_items)) = item_mod.content {
                    for mod_item in mod_items {
                        self.remove_doc_attributes(mod_item);
                        self.remove_doc_from_children(mod_item);
                    }
                }
            }
            _ => {}
        }
    }

    /// Remove documentation from struct/enum fields
    fn remove_docs_from_fields(&self, fields: &mut syn::Fields) {
        match fields {
            syn::Fields::Named(fields) => {
                for field in &mut fields.named {
                    field.attrs.retain(|attr| !self.is_doc_attribute(attr));
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &mut fields.unnamed {
                    field.attrs.retain(|attr| !self.is_doc_attribute(attr));
                }
            }
            syn::Fields::Unit => {}
        }
    }

    /// Remove documentation from function inputs
    fn remove_docs_from_fn_inputs(
        &self,
        inputs: &mut syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) {
        for input in inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                pat_type.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
        }
    }

    /// Remove documentation from impl items
    fn remove_docs_from_impl_item(&self, impl_item: &mut syn::ImplItem) {
        match impl_item {
            syn::ImplItem::Fn(method) => {
                method.attrs.retain(|attr| !self.is_doc_attribute(attr));
                self.remove_docs_from_fn_inputs(&mut method.sig.inputs);
            }
            syn::ImplItem::Const(const_item) => {
                const_item.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            syn::ImplItem::Type(type_item) => {
                type_item.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            _ => {}
        }
    }

    /// Remove documentation from trait items
    fn remove_docs_from_trait_item(&self, trait_item: &mut syn::TraitItem) {
        match trait_item {
            syn::TraitItem::Fn(method) => {
                method.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            syn::TraitItem::Const(const_item) => {
                const_item.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            syn::TraitItem::Type(type_item) => {
                type_item.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            _ => {}
        }
    }

    /// Check if an item has test attributes
    fn has_test_attribute(&self, item: &syn::Item) -> bool {
        let attrs = match item {
            syn::Item::Fn(item_fn) => &item_fn.attrs,
            syn::Item::Mod(item_mod) => &item_mod.attrs,
            syn::Item::Struct(item_struct) => &item_struct.attrs,
            syn::Item::Enum(item_enum) => &item_enum.attrs,
            syn::Item::Trait(item_trait) => &item_trait.attrs,
            syn::Item::Impl(item_impl) => &item_impl.attrs,
            _ => return false,
        };

        attrs.iter().any(|attr| {
            if attr.path().is_ident("test") {
                return true;
            }

            if attr.path().is_ident("cfg") {
                let attr_str = quote::quote!(#attr).to_string();
                return attr_str.contains("test");
            }

            false
        })
    }

    /// Remove documentation attributes from an item
    fn remove_doc_attributes(&self, item: &mut syn::Item) {
        let attrs = match item {
            syn::Item::Fn(item_fn) => &mut item_fn.attrs,
            syn::Item::Mod(item_mod) => &mut item_mod.attrs,
            syn::Item::Struct(item_struct) => &mut item_struct.attrs,
            syn::Item::Enum(item_enum) => &mut item_enum.attrs,
            syn::Item::Trait(item_trait) => &mut item_trait.attrs,
            syn::Item::Impl(item_impl) => &mut item_impl.attrs,
            syn::Item::Type(item_type) => &mut item_type.attrs,
            syn::Item::Const(item_const) => &mut item_const.attrs,
            syn::Item::Static(item_static) => &mut item_static.attrs,
            syn::Item::Use(item_use) => &mut item_use.attrs,
            syn::Item::ExternCrate(item_extern_crate) => &mut item_extern_crate.attrs,
            _ => return,
        };

        attrs.retain(|attr| !self.is_doc_attribute(attr));
    }

    /// Expand extern crate declarations
    fn expand_extern_crate(&self, items: &mut Vec<syn::Item>) -> Result<()> {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if self.is_extern_crate(&item, self.crate_name) {
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
                    message: format!("Failed to parse lib.rs: {}", e),
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

    /// Expand use paths
    fn expand_use_path(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if !self.is_use_path(&item, self.crate_name) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    /// Expand module declarations
    fn expand_mods(&mut self, item: &mut syn::ItemMod) -> Result<()> {
        if item.content.is_some() {
            return Ok(());
        }

        let name = item.ident.to_string();
        let (base_path, code) = FileManager::find_module_file(self.base_path, &name)?;

        let mut file = syn::parse_file(&code).map_err(|e| BundlerError::Parsing {
            message: format!("Failed to parse module file: {}", e),
            file_path: Some(base_path.join(format!("{}.rs", name))),
        })?;

        // Use the original config for expansion to ensure consistent behavior
        let mut expander = CodeTransformer::new(&base_path, self.crate_name, self.config.clone());

        // Apply full transformation to the module content
        expander.expand_items(&mut file.items)?;

        // Also visit each item to handle nested modules with the correct base path
        for item in &mut file.items {
            expander.visit_item_mut(item);
        }

        item.content = Some((Default::default(), file.items));
        Ok(())
    }

    /// Expand crate paths
    fn expand_crate_path(&mut self, path: &mut syn::Path) {
        if self.path_starts_with(path, self.crate_name) {
            let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                .into_pairs()
                .skip(1)
                .collect();
            path.segments = new_segments;
        }
    }

    /// Check if item is an extern crate declaration
    fn is_extern_crate(&self, item: &syn::Item, crate_name: &str) -> bool {
        if let syn::Item::ExternCrate(ref item) = *item {
            if item.ident == crate_name {
                return true;
            }
        }
        false
    }

    /// Check if path starts with a specific segment
    fn path_starts_with(&self, path: &syn::Path, segment: &str) -> bool {
        if let Some(el) = path.segments.first() {
            if el.ident == segment {
                return true;
            }
        }
        false
    }

    /// Check if item is a use path
    fn is_use_path(&self, item: &syn::Item, first_segment: &str) -> bool {
        if let syn::Item::Use(ref item) = *item {
            if let syn::UseTree::Path(ref path) = item.tree {
                if path.ident == first_segment {
                    return true;
                }
            }
        }
        false
    }
}

impl<'a> VisitMut for CodeTransformer<'a> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        if self.config.remove_docs {
            file.attrs.retain(|attr| !self.is_doc_attribute(attr));
        }

        for attr in &mut file.attrs {
            self.visit_attribute_mut(attr);
        }

        if let Err(e) = self.expand_items(&mut file.items) {
            eprintln!("Warning: Failed to expand items: {}", e);
        }

        for item in &mut file.items {
            self.visit_item_mut(item);
        }
    }

    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        for attr in &mut item.attrs {
            self.visit_attribute_mut(attr);
        }
        self.visit_visibility_mut(&mut item.vis);
        self.visit_ident_mut(&mut item.ident);

        if let Err(e) = self.expand_mods(item) {
            eprintln!("Warning: Failed to expand module {}: {}", item.ident, e);
        }

        // Note: We don't recursively visit the expanded content here because
        // expand_mods already handles the full transformation of the module content
        // with the correct base path context
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let segment = el.value_mut();
            self.visit_path_segment_mut(segment);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_transform_config_default() {
        let config = TransformConfig::default();
        assert!(config.remove_tests);
        assert!(config.remove_docs);
        assert!(config.expand_modules);
    }

    #[test]
    fn test_is_doc_attribute() {
        let base_path = PathBuf::from("/tmp");
        let transformer =
            CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

        // Test with a doc attribute
        let doc_attr: syn::Attribute = syn::parse_quote!(#[doc = "test"]);
        assert!(transformer.is_doc_attribute(&doc_attr));

        // Test with a non-doc attribute
        let non_doc_attr: syn::Attribute = syn::parse_quote!(#[test]);
        assert!(!transformer.is_doc_attribute(&non_doc_attr));
    }

    #[test]
    fn test_has_test_attribute() {
        let base_path = PathBuf::from("/tmp");
        let transformer =
            CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

        // Test function with test attribute
        let test_fn: syn::Item = syn::parse_quote! {
            #[test]
            fn test_function() {}
        };
        assert!(transformer.has_test_attribute(&test_fn));

        // Test regular function
        let regular_fn: syn::Item = syn::parse_quote! {
            fn regular_function() {}
        };
        assert!(!transformer.has_test_attribute(&regular_fn));
    }

    #[test]
    fn test_is_extern_crate() {
        let base_path = PathBuf::from("/tmp");
        let transformer =
            CodeTransformer::new(&base_path, "test_crate", TransformConfig::default());

        // Test extern crate with matching name
        let extern_crate_item: syn::Item = syn::parse_quote! {
            extern crate test_crate;
        };
        assert!(transformer.is_extern_crate(&extern_crate_item, "test_crate"));

        // Test extern crate with different name
        assert!(!transformer.is_extern_crate(&extern_crate_item, "other_crate"));

        // Test non-extern-crate item
        let fn_item: syn::Item = syn::parse_quote! {
            fn test() {}
        };
        assert!(!transformer.is_extern_crate(&fn_item, "test_crate"));
    }
}
