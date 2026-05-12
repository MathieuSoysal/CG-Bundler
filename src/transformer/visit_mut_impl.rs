use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

use super::CodeTransformer;

impl VisitMut for CodeTransformer<'_> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        if self.config.remove_docs {
            file.attrs.retain(|attr| !Self::is_doc_attribute(attr));
        }

        for attr in &mut file.attrs {
            self.visit_attribute_mut(attr);
        }

        if let Err(e) = self.expand_items(&mut file.items) {
            eprintln!("Warning: Failed to expand items: {e}");
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
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let segment = el.value_mut();
            self.visit_path_segment_mut(segment);
        }
    }
}
