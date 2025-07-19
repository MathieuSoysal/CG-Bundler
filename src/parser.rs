//! Code parsing implementation using syn for AST manipulation

use syn::{
    parse_file, Attribute, Item, ItemFn, ItemMod, Meta, AttrStyle,
    visit_mut::{self, VisitMut}
};
use crate::error::{ProcessingError, Result};
use crate::syntax_tree::SyntaxTree;
use crate::traits::CodeParser;

/// Code parser implementation using syn
pub struct SynCodeParser;

impl SynCodeParser {
    pub fn new() -> Self {
        Self
    }
    
    /// Remove comments by filtering out doc comments and other comment attributes
    fn remove_comments(&self, tree: &mut SyntaxTree) {
        let mut visitor = CommentRemover;
        visitor.visit_file_mut(tree.file_mut());
    }
    
    /// Remove test code including functions and modules marked with #[test] or #[cfg(test)]
    fn remove_test_code(&self, tree: &mut SyntaxTree) {
        let items = tree.items_mut();
        items.retain(|item| !self.is_test_item(item));
        
        // Also remove test code from remaining items
        let mut visitor = TestCodeRemover;
        visitor.visit_file_mut(tree.file_mut());
    }
    
    /// Remove #[cfg(test)] attributes and similar conditional compilation for tests
    fn remove_cfg_test_attributes(&self, tree: &mut SyntaxTree) {
        let mut visitor = CfgTestRemover;
        visitor.visit_file_mut(tree.file_mut());
    }
    
    /// Check if an item is test-related
    fn is_test_item(&self, item: &Item) -> bool {
        match item {
            Item::Fn(func) => self.is_test_function(func),
            Item::Mod(module) => self.is_test_module(module),
            _ => false,
        }
    }
    
    /// Check if a function is a test function
    fn is_test_function(&self, func: &ItemFn) -> bool {
        func.attrs.iter().any(|attr| self.is_test_attribute(attr))
    }
    
    /// Check if a module is a test module
    fn is_test_module(&self, module: &ItemMod) -> bool {
        module.attrs.iter().any(|attr| self.is_cfg_test_attribute(attr))
    }
    
    /// Check if an attribute is #[test]
    fn is_test_attribute(&self, attr: &Attribute) -> bool {
        if let Meta::Path(path) = &attr.meta {
            path.is_ident("test")
        } else {
            false
        }
    }
    
    /// Check if an attribute is #[cfg(test)]
    fn is_cfg_test_attribute(&self, attr: &Attribute) -> bool {
        if let Meta::List(list) = &attr.meta {
            if list.path.is_ident("cfg") {
                return list.tokens.to_string().contains("test");
            }
        }
        false
    }
    
    /// Check if an attribute is a doc comment
    #[allow(dead_code)]
    fn is_doc_comment_attribute(&self, attr: &Attribute) -> bool {
        matches!(attr.style, AttrStyle::Outer) && 
        attr.path().is_ident("doc")
    }
}

impl CodeParser for SynCodeParser {
    fn parse(&self, content: &str) -> Result<SyntaxTree> {
        let file = parse_file(content)
            .map_err(|e| ProcessingError::ParseError(e.to_string()))?;
        Ok(SyntaxTree::new(file))
    }
    
    fn remove_unwanted_elements(&self, tree: &mut SyntaxTree) -> Result<()> {
        // Remove test code first
        self.remove_test_code(tree);
        
        // Remove comments and doc comments (CommentRemover now handles both)
        self.remove_comments(tree);
        
        // Remove cfg(test) attributes
        self.remove_cfg_test_attributes(tree);
        
        Ok(())
    }
}

impl Default for SynCodeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor to remove comment attributes
struct CommentRemover;

impl VisitMut for CommentRemover {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        // Remove inner doc attributes from the file
        file.attrs.retain(|attr| !self.is_doc_attribute(attr));
        visit_mut::visit_file_mut(self, file);
    }
    
    fn visit_item_mut(&mut self, item: &mut Item) {
        // Remove doc comments from items
        match item {
            Item::Fn(func) => {
                func.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Struct(s) => {
                s.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Enum(e) => {
                e.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Mod(m) => {
                m.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Use(u) => {
                u.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Type(t) => {
                t.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Const(c) => {
                c.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Static(s) => {
                s.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Trait(t) => {
                t.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            Item::Impl(i) => {
                i.attrs.retain(|attr| !self.is_doc_attribute(attr));
            }
            _ => {}
        }
        
        visit_mut::visit_item_mut(self, item);
    }
    
    fn visit_field_mut(&mut self, field: &mut syn::Field) {
        // Remove doc attributes from struct/enum fields
        field.attrs.retain(|attr| !self.is_doc_attribute(attr));
        visit_mut::visit_field_mut(self, field);
    }
    
    fn visit_variant_mut(&mut self, variant: &mut syn::Variant) {
        // Remove doc attributes from enum variants
        variant.attrs.retain(|attr| !self.is_doc_attribute(attr));
        visit_mut::visit_variant_mut(self, variant);
    }
}

impl CommentRemover {
    /// Check if an attribute is a documentation attribute
    fn is_doc_attribute(&self, attr: &Attribute) -> bool {
        // Check for #[doc = "..."] attributes
        if attr.path().is_ident("doc") {
            return true;
        }
        
        // Check for outer doc comments (///), which become #[doc = "..."]
        if let AttrStyle::Outer = attr.style {
            if attr.path().is_ident("doc") {
                return true;
            }
        }
        
        // Check for inner doc comments (//!), which become #![doc = "..."]
        if let AttrStyle::Inner(_) = attr.style {
            if attr.path().is_ident("doc") {
                return true;
            }
        }
        
        false
    }
}

/// Visitor to remove test code
struct TestCodeRemover;

impl VisitMut for TestCodeRemover {
    fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
        if let Some((_, items)) = &mut module.content {
            items.retain(|item| {
                match item {
                    Item::Fn(func) => !func.attrs.iter().any(|attr| {
                        if let Meta::Path(path) = &attr.meta {
                            path.is_ident("test") || path.is_ident("bench")
                        } else {
                            false
                        }
                    }),
                    _ => true,
                }
            });
        }
        
        visit_mut::visit_item_mod_mut(self, module);
    }
}

/// Visitor to remove #[cfg(test)] attributes
struct CfgTestRemover;

impl VisitMut for CfgTestRemover {
    fn visit_item_mut(&mut self, item: &mut Item) {
        match item {
            Item::Fn(func) => {
                func.attrs.retain(|attr| !is_cfg_test_attr(attr));
            }
            Item::Mod(m) => {
                m.attrs.retain(|attr| !is_cfg_test_attr(attr));
            }
            Item::Struct(s) => {
                s.attrs.retain(|attr| !is_cfg_test_attr(attr));
            }
            Item::Enum(e) => {
                e.attrs.retain(|attr| !is_cfg_test_attr(attr));
            }
            _ => {}
        }
        
        visit_mut::visit_item_mut(self, item);
    }
}

fn is_cfg_test_attr(attr: &Attribute) -> bool {
    if let Meta::List(list) = &attr.meta {
        if list.path.is_ident("cfg") {
            return list.tokens.to_string().contains("test");
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() -> Result<()> {
        let parser = SynCodeParser::new();
        let code = "fn main() { println!(\"Hello, world!\"); }";
        
        let tree = parser.parse(code)?;
        assert_eq!(tree.items().len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_parse_invalid_code() {
        let parser = SynCodeParser::new();
        let code = "fn main( { invalid syntax }";
        
        let result = parser.parse(code);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProcessingError::ParseError(_)));
    }
    
    #[test]
    fn test_remove_test_function() -> Result<()> {
        let parser = SynCodeParser::new();
        let code = r#"
            fn main() {}
            
            #[test]
            fn test_something() {}
            
            fn other_function() {}
        "#;
        
        let mut tree = parser.parse(code)?;
        parser.remove_unwanted_elements(&mut tree)?;
        
        // Should only have main and other_function, not the test
        assert_eq!(tree.items().len(), 2);
        
        Ok(())
    }
    
    #[test]
    fn test_remove_doc_comments() -> Result<()> {
        let parser = SynCodeParser::new();
        let code = r#"
            /// This is a doc comment
            fn main() {}
        "#;
        
        let mut tree = parser.parse(code)?;
        parser.remove_unwanted_elements(&mut tree)?;
        
        // Function should still exist but without doc comments
        assert_eq!(tree.items().len(), 1);
        if let Item::Fn(func) = &tree.items()[0] {
            assert!(func.attrs.iter().all(|attr| !attr.path().is_ident("doc")));
        }
        
        Ok(())
    }
    
    #[test]
    fn test_remove_cfg_test_module() -> Result<()> {
        let parser = SynCodeParser::new();
        let code = r#"
            fn main() {}
            
            #[cfg(test)]
            mod tests {
                fn test_something() {}
            }
        "#;
        
        let mut tree = parser.parse(code)?;
        parser.remove_unwanted_elements(&mut tree)?;
        
        // Should only have main function, test module should be removed
        assert_eq!(tree.items().len(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_is_test_attribute() {
        let parser = SynCodeParser::new();
        
        // Parse a test function to get the attribute
        let code = "#[test] fn test_func() {}";
        let file = syn::parse_file(code).unwrap();
        
        if let Item::Fn(func) = &file.items[0] {
            assert!(parser.is_test_attribute(&func.attrs[0]));
        }
    }
}
