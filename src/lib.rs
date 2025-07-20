//! See [README.md](https://github.com/slava-sh/rust-bundler/blob/master/README.md)

use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;

use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

/// Creates a single-source-file version of a Cargo package.
pub fn bundle<P: AsRef<Path>>(package_path: P) -> String {
    let manifest_path = package_path.as_ref().join("Cargo.toml");
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .expect("failed to obtain cargo metadata");
    
    // Find the root package (the one we're bundling)
    let root_package = metadata.packages.iter()
        .find(|pkg| {
            // Compare canonical paths to handle relative vs absolute paths
            std::fs::canonicalize(&pkg.manifest_path).unwrap_or_else(|_| pkg.manifest_path.clone().into())
                == std::fs::canonicalize(&manifest_path).unwrap_or_else(|_| manifest_path.clone())
        })
        .expect("failed to find root package in metadata");
    
    let targets = &root_package.targets;
    let bins: Vec<_> = targets.iter().filter(|t| target_is(t, "bin")).collect();
    assert!(bins.len() != 0, "no binary target found");
    assert!(bins.len() == 1, "multiple binary targets not supported");
    let bin = bins[0];
    let libs: Vec<_> = targets.iter().filter(|t| target_is(t, "lib")).collect();
    assert!(libs.len() <= 1, "multiple library targets not supported");
    let lib = libs.get(0).unwrap_or(&bin);
    let base_path = Path::new(&lib.src_path)
        .parent()
        .expect("lib.src_path has no parent");
    let crate_name = &lib.name;
    let code = read_file(&Path::new(&bin.src_path)).expect("failed to read binary target source");
    let mut file = syn::parse_file(&code).expect("failed to parse binary target source");
    Expander {
        base_path,
        crate_name,
    }.visit_file_mut(&mut file);
    quote::quote!(#file).to_string()
}

fn target_is(target: &cargo_metadata::Target, target_kind: &str) -> bool {
    target.kind.iter().any(|kind| kind == target_kind)
}

struct Expander<'a> {
    base_path: &'a Path,
    crate_name: &'a str,
}

impl<'a> Expander<'a> {
    fn expand_items(&self, items: &mut Vec<syn::Item>) {
        self.expand_extern_crate(items);
        self.expand_use_path(items);
        self.filter_tests_and_docs(items);
    }

    fn filter_tests_and_docs(&self, items: &mut Vec<syn::Item>) {
        items.retain(|item| {
            // Remove items with #[cfg(test)] or #[test] attributes
            !self.has_test_attribute(item)
        });
        
        // Remove documentation comments from remaining items and their children
        for item in items.iter_mut() {
            self.remove_doc_attributes(item);
            self.remove_doc_from_children(item);
        }
    }

    fn is_doc_attribute(&self, attr: &syn::Attribute) -> bool {
        // Check if it's a doc attribute by path
        if attr.path().is_ident("doc") {
            return true;
        }
        
        // Also check the string representation for any remaining doc attributes
        let attr_str = quote::quote!(#attr).to_string();
        
        // Filter out various forms of doc attributes
        attr_str.starts_with("# [doc") || 
        attr_str.starts_with("#[doc") || 
        attr_str.contains("doc =")
    }

    fn remove_doc_from_children(&self, item: &mut syn::Item) {
        match item {
            syn::Item::Struct(item_struct) => {
                // Remove docs from struct fields
                if let syn::Fields::Named(fields) = &mut item_struct.fields {
                    for field in &mut fields.named {
                        field.attrs.retain(|attr| !self.is_doc_attribute(attr));
                    }
                }
                if let syn::Fields::Unnamed(fields) = &mut item_struct.fields {
                    for field in &mut fields.unnamed {
                        field.attrs.retain(|attr| !self.is_doc_attribute(attr));
                    }
                }
            }
            syn::Item::Enum(item_enum) => {
                // Remove docs from enum variants
                for variant in &mut item_enum.variants {
                    variant.attrs.retain(|attr| !self.is_doc_attribute(attr));
                    // Also remove docs from variant fields
                    match &mut variant.fields {
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
            }
            syn::Item::Fn(item_fn) => {
                // Remove docs from function parameters
                for input in &mut item_fn.sig.inputs {
                    if let syn::FnArg::Typed(pat_type) = input {
                        pat_type.attrs.retain(|attr| !self.is_doc_attribute(attr));
                    }
                }
            }
            syn::Item::Impl(item_impl) => {
                // Remove docs from impl methods
                for impl_item in &mut item_impl.items {
                    match impl_item {
                        syn::ImplItem::Fn(method) => {
                            method.attrs.retain(|attr| !self.is_doc_attribute(attr));
                            // Remove docs from method parameters
                            for input in &mut method.sig.inputs {
                                if let syn::FnArg::Typed(pat_type) = input {
                                    pat_type.attrs.retain(|attr| !self.is_doc_attribute(attr));
                                }
                            }
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
            }
            syn::Item::Trait(item_trait) => {
                // Remove docs from trait methods
                for trait_item in &mut item_trait.items {
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
            }
            syn::Item::Mod(item_mod) => {
                // Remove docs from inline module content
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
            // Check for #[test]
            if attr.path().is_ident("test") {
                return true;
            }
            
            // Check for #[cfg(test)] - simplified approach for syn 2.0
            if attr.path().is_ident("cfg") {
                // Convert attribute to string and check if it contains "test"
                let attr_str = quote::quote!(#attr).to_string();
                return attr_str.contains("test");
            }
            
            false
        })
    }

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

        // Remove doc comments (/// and //!) and #[doc = "..."] attributes
        attrs.retain(|attr| !self.is_doc_attribute(attr));
    }

    fn expand_extern_crate(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if is_extern_crate(&item, self.crate_name) {
                eprintln!(
                    "expanding crate {} in {}",
                    self.crate_name,
                    self.base_path.to_str().unwrap()
                );
                let code =
                    read_file(&self.base_path.join("lib.rs")).expect("failed to read lib.rs");
                let lib = syn::parse_file(&code).expect("failed to parse lib.rs");
                new_items.extend(lib.items);
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_use_path(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if !is_use_path(&item, self.crate_name) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_mods(&self, item: &mut syn::ItemMod) {
        if item.content.is_some() {
            return;
        }
        let name = item.ident.to_string();
        let other_base_path = self.base_path.join(&name);
        let (base_path, code) = vec![
            (self.base_path, format!("{}.rs", name)),
            (&other_base_path, String::from("mod.rs")),
        ].into_iter()
            .flat_map(|(base_path, file_name)| {
                read_file(&base_path.join(file_name)).map(|code| (base_path, code))
            })
            .next()
            .expect("mod not found");
        eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
        let mut file = syn::parse_file(&code).expect("failed to parse file");
        
        // Create a new expander and apply filtering to the module content
        let mut expander = Expander {
            base_path,
            crate_name: self.crate_name,
        };
        expander.visit_file_mut(&mut file);
        
        item.content = Some((Default::default(), file.items));
    }

    fn expand_crate_path(&mut self, path: &mut syn::Path) {
        if path_starts_with(path, self.crate_name) {
            let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                .into_pairs()
                .skip(1)
                .collect();
            path.segments = new_segments;
        }
    }
}

impl<'a> VisitMut for Expander<'a> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        // Remove doc comments from file level attributes
        file.attrs.retain(|attr| !self.is_doc_attribute(attr));
        
        for it in &mut file.attrs {
            self.visit_attribute_mut(it)
        }
        self.expand_items(&mut file.items);
        for it in &mut file.items {
            self.visit_item_mut(it)
        }
    }

    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        for it in &mut item.attrs {
            self.visit_attribute_mut(it)
        }
        self.visit_visibility_mut(&mut item.vis);
        self.visit_ident_mut(&mut item.ident);
        self.expand_mods(item);
        if let Some(ref mut it) = item.content {
            for it in &mut (it).1 {
                self.visit_item_mut(it);
            }
        }
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let it = el.value_mut();
            self.visit_path_segment_mut(it)
        }
    }
}

fn is_extern_crate(item: &syn::Item, crate_name: &str) -> bool {
    if let syn::Item::ExternCrate(ref item) = *item {
        if item.ident == crate_name {
            return true;
        }
    }
    false
}

fn path_starts_with(path: &syn::Path, segment: &str) -> bool {
    if let Some(el) = path.segments.first() {
        if el.ident == segment {
            return true;
        }
    }
    false
}

fn is_use_path(item: &syn::Item, first_segment: &str) -> bool {
    if let syn::Item::Use(ref item) = *item {
        if let syn::UseTree::Path(ref path) = item.tree {
            if path.ident == first_segment {
                return true;
            }
        }
    }
    false
}

fn read_file(path: &Path) -> Option<String> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    Some(buf)
}
