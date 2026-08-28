use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

use super::CodeTransformer;
use super::macro_paths::{RootRewrite, rewrite_root_in_tokens};

impl VisitMut for CodeTransformer<'_> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        if self.config.remove_docs {
            file.attrs.retain(|attr| !Self::is_doc_attribute(attr));
        }

        for attr in &mut file.attrs {
            self.visit_attribute_mut(attr);
        }

        if let Err(e) = self.expand_items(&mut file.items) {
            self.record_error(e);
            return;
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

        if item.content.is_none() {
            // `expand_mods` fully transforms the loaded module with the right base path.
            if let Err(e) = self.expand_mods(item) {
                self.record_error(e);
            }
            return;
        }

        // Inline `mod x { .. }`: submodule files of `x` live in `<base_path>/x/`.
        let child_base = self.base_path.join(item.ident.to_string());
        let mut child = self.child(&child_base);

        if let Some((_, items)) = item.content.as_mut() {
            if let Err(e) = child.expand_items(items) {
                child.record_error(e);
            } else {
                for nested in items.iter_mut() {
                    child.visit_item_mut(nested);
                }
            }
        }

        if let Some(error) = child.error.take() {
            self.record_error(error);
        }
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let segment = el.value_mut();
            self.visit_path_segment_mut(segment);
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        self.visit_path_mut(&mut mac.path);

        if !self.config.expand_modules {
            return;
        }

        let rewrite = if self.is_root {
            RootRewrite::Strip
        } else {
            RootRewrite::Rename
        };
        let mut tokens =
            rewrite_root_in_tokens(std::mem::take(&mut mac.tokens), self.crate_name, &rewrite);

        if !self.is_root {
            // Dependencies are inlined as `mod <name>` at the bundle root.
            let mut names: Vec<&String> = self.external_libs.keys().collect();
            names.sort_unstable();
            for name in names {
                tokens = rewrite_root_in_tokens(tokens, name, &RootRewrite::PrefixCrate);
            }
        }

        mac.tokens = tokens;
    }
}
