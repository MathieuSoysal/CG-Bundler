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
    ///
    /// Every item kind that can carry an attribute is inspected. Restricting this
    /// to the item kinds that *define* code is not enough: a surviving
    /// `#[cfg(test)] use some_dep::Helper;` makes dependency detection inline the
    /// whole of `some_dep` into a bundle that never uses it.
    #[must_use]
    pub fn has_test_attribute(item: &syn::Item) -> bool {
        Self::item_attributes(item).is_some_and(|attrs| attrs.iter().any(Self::is_test_attribute))
    }

    /// The attributes of `item`, for every item kind that can carry them.
    fn item_attributes(item: &syn::Item) -> Option<&[syn::Attribute]> {
        let attrs = match item {
            syn::Item::Const(inner) => &inner.attrs,
            syn::Item::Enum(inner) => &inner.attrs,
            syn::Item::ExternCrate(inner) => &inner.attrs,
            syn::Item::Fn(inner) => &inner.attrs,
            syn::Item::ForeignMod(inner) => &inner.attrs,
            syn::Item::Impl(inner) => &inner.attrs,
            syn::Item::Macro(inner) => &inner.attrs,
            syn::Item::Mod(inner) => &inner.attrs,
            syn::Item::Static(inner) => &inner.attrs,
            syn::Item::Struct(inner) => &inner.attrs,
            syn::Item::Trait(inner) => &inner.attrs,
            syn::Item::TraitAlias(inner) => &inner.attrs,
            syn::Item::Type(inner) => &inner.attrs,
            syn::Item::Union(inner) => &inner.attrs,
            syn::Item::Use(inner) => &inner.attrs,
            _ => return None,
        };

        Some(attrs)
    }

    /// Whether an attribute marks an item as test-only.
    ///
    /// Recognises `#[test]` / `#[<path>::test]` and `cfg` predicates that require `test`
    /// to be enabled. `#[cfg(not(test))]` and unrelated predicates such as
    /// `#[cfg(feature = "fastest")]` are deliberately not treated as test markers.
    #[must_use]
    pub fn is_test_attribute(attr: &syn::Attribute) -> bool {
        let path = attr.path();

        if path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
        {
            return true;
        }

        if path.is_ident("cfg") {
            return attr
                .parse_args::<syn::Meta>()
                .is_ok_and(|meta| Self::cfg_requires_test(&meta, false));
        }

        false
    }

    /// Evaluate whether a `cfg` predicate can only hold when `test` is enabled.
    fn cfg_requires_test(meta: &syn::Meta, negated: bool) -> bool {
        match meta {
            syn::Meta::Path(path) => !negated && path.is_ident("test"),
            syn::Meta::List(list) => {
                let is_all = list.path.is_ident("all");
                let is_any = list.path.is_ident("any");
                let is_not = list.path.is_ident("not");
                if !is_all && !is_any && !is_not {
                    return false;
                }

                let negated = if is_not { !negated } else { negated };

                // `all(..)` holds only if every branch holds, so one test-only
                // branch is enough to make the whole predicate test-only.
                // `any(..)` holds if *some* branch holds, so it is test-only only
                // when every branch is. Negation swaps the two (De Morgan), and
                // `not(..)` takes a single argument either way.
                let requires_every_branch = !is_not && (is_any != negated);

                list.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|nested| {
                    if requires_every_branch {
                        !nested.is_empty()
                            && nested
                                .iter()
                                .all(|inner| Self::cfg_requires_test(inner, negated))
                    } else {
                        nested
                            .iter()
                            .any(|inner| Self::cfg_requires_test(inner, negated))
                    }
                })
            }
            syn::Meta::NameValue(_) => false,
        }
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
