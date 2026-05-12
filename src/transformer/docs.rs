use super::CodeTransformer;

impl CodeTransformer<'_> {
    /// Remove file-level documentation.
    pub(super) fn remove_file_level_docs(&self, file: &mut syn::File) {
        if self.config.remove_docs {
            file.attrs.retain(|attr| !Self::is_doc_attribute(attr));
        }
    }

    /// Filter out tests and documentation.
    pub(super) fn filter_tests_and_docs(&self, items: &mut Vec<syn::Item>) {
        items.retain(|item| {
            if self.config.remove_tests && Self::has_test_attribute(item) {
                return false;
            }
            true
        });

        if self.config.remove_docs {
            for item in items.iter_mut() {
                Self::remove_doc_attributes(item);
                Self::remove_doc_from_children(item);
            }
        }
    }

    /// Check if an attribute is a documentation attribute.
    #[must_use]
    pub fn is_doc_attribute(attr: &syn::Attribute) -> bool {
        if attr.path().is_ident("doc") {
            return true;
        }

        let attr_str = quote::quote!(#attr).to_string();
        attr_str.starts_with("# [doc")
            || attr_str.starts_with("#[doc")
            || attr_str.contains("doc =")
    }

    /// Remove documentation from child elements.
    fn remove_doc_from_children(item: &mut syn::Item) {
        match item {
            syn::Item::Struct(item_struct) => {
                Self::remove_docs_from_fields(&mut item_struct.fields);
            }
            syn::Item::Enum(item_enum) => {
                for variant in &mut item_enum.variants {
                    variant.attrs.retain(|attr| !Self::is_doc_attribute(attr));
                    Self::remove_docs_from_fields(&mut variant.fields);
                }
            }
            syn::Item::Fn(item_fn) => {
                Self::remove_docs_from_fn_inputs(&mut item_fn.sig.inputs);
            }
            syn::Item::Impl(item_impl) => {
                for impl_item in &mut item_impl.items {
                    Self::remove_docs_from_impl_item(impl_item);
                }
            }
            syn::Item::Trait(item_trait) => {
                for trait_item in &mut item_trait.items {
                    Self::remove_docs_from_trait_item(trait_item);
                }
            }
            syn::Item::Mod(item_mod) => {
                if let Some((_, ref mut mod_items)) = item_mod.content {
                    for mod_item in mod_items {
                        Self::remove_doc_attributes(mod_item);
                        Self::remove_doc_from_children(mod_item);
                    }
                }
            }
            _ => {}
        }
    }

    /// Remove documentation from struct/enum fields.
    fn remove_docs_from_fields(fields: &mut syn::Fields) {
        match fields {
            syn::Fields::Named(fields) => {
                for field in &mut fields.named {
                    field.attrs.retain(|attr| !Self::is_doc_attribute(attr));
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &mut fields.unnamed {
                    field.attrs.retain(|attr| !Self::is_doc_attribute(attr));
                }
            }
            syn::Fields::Unit => {}
        }
    }

    /// Remove documentation from function inputs.
    fn remove_docs_from_fn_inputs(
        inputs: &mut syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ) {
        for input in inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                pat_type.attrs.retain(|attr| !Self::is_doc_attribute(attr));
            }
        }
    }

    /// Remove documentation from impl items.
    fn remove_docs_from_impl_item(impl_item: &mut syn::ImplItem) {
        match impl_item {
            syn::ImplItem::Fn(method) => {
                method.attrs.retain(|attr| !Self::is_doc_attribute(attr));
                Self::remove_docs_from_fn_inputs(&mut method.sig.inputs);
            }
            syn::ImplItem::Const(const_item) => {
                const_item
                    .attrs
                    .retain(|attr| !Self::is_doc_attribute(attr));
            }
            syn::ImplItem::Type(type_item) => {
                type_item.attrs.retain(|attr| !Self::is_doc_attribute(attr));
            }
            _ => {}
        }
    }

    /// Remove documentation from trait items.
    fn remove_docs_from_trait_item(trait_item: &mut syn::TraitItem) {
        match trait_item {
            syn::TraitItem::Fn(method) => {
                method.attrs.retain(|attr| !Self::is_doc_attribute(attr));
            }
            syn::TraitItem::Const(const_item) => {
                const_item
                    .attrs
                    .retain(|attr| !Self::is_doc_attribute(attr));
            }
            syn::TraitItem::Type(type_item) => {
                type_item.attrs.retain(|attr| !Self::is_doc_attribute(attr));
            }
            _ => {}
        }
    }

    /// Check if an item has test attributes.
    #[must_use]
    pub fn has_test_attribute(item: &syn::Item) -> bool {
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

    /// Remove documentation attributes from an item.
    fn remove_doc_attributes(item: &mut syn::Item) {
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

        attrs.retain(|attr| !Self::is_doc_attribute(attr));
    }
}
