//! Syntax tree representation for parsed Rust code

use syn::{File, Item};
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Wrapper around syn::File with additional functionality
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    file: File,
}

impl SyntaxTree {
    /// Create a new syntax tree from a syn::File
    pub fn new(file: File) -> Self {
        Self { file }
    }
    
    /// Create an empty syntax tree
    pub fn empty() -> Self {
        Self {
            file: File {
                shebang: None,
                attrs: Vec::new(),
                items: Vec::new(),
            }
        }
    }
    
    /// Get the items in the syntax tree
    pub fn items(&self) -> &Vec<Item> {
        &self.file.items
    }
    
    /// Get mutable reference to items
    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.file.items
    }
    
    /// Add an item to the syntax tree
    pub fn add_item(&mut self, item: Item) {
        self.file.items.push(item);
    }
    
    /// Remove an item at the specified index
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.file.items.len() {
            Some(self.file.items.remove(index))
        } else {
            None
        }
    }
    
    /// Convert to token stream for code generation
    pub fn to_token_stream(&self) -> TokenStream {
        self.file.to_token_stream()
    }
    
    /// Get reference to the underlying syn::File
    pub fn file(&self) -> &File {
        &self.file
    }
    
    /// Get mutable reference to the underlying syn::File
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl From<File> for SyntaxTree {
    fn from(file: File) -> Self {
        Self::new(file)
    }
}

impl Into<File> for SyntaxTree {
    fn into(self) -> File {
        self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Item};

    #[test]
    fn test_empty_syntax_tree() {
        let tree = SyntaxTree::empty();
        assert_eq!(tree.items().len(), 0);
    }

    #[test]
    fn test_add_item() {
        let mut tree = SyntaxTree::empty();
        let item: Item = parse_quote! {
            fn test() {}
        };
        tree.add_item(item);
        assert_eq!(tree.items().len(), 1);
    }

    #[test]
    fn test_remove_item() {
        let mut tree = SyntaxTree::empty();
        let item: Item = parse_quote! {
            fn test() {}
        };
        tree.add_item(item);
        
        let removed = tree.remove_item(0);
        assert!(removed.is_some());
        assert_eq!(tree.items().len(), 0);
    }

    #[test]
    fn test_remove_invalid_index() {
        let mut tree = SyntaxTree::empty();
        let removed = tree.remove_item(5);
        assert!(removed.is_none());
    }

    #[test]
    fn test_to_token_stream() {
        let mut tree = SyntaxTree::empty();
        let item: Item = parse_quote! {
            fn test() { println!("hello"); }
        };
        tree.add_item(item);
        
        let tokens = tree.to_token_stream();
        let code = tokens.to_string();
        assert!(code.contains("fn test"));
        // The exact format might vary, so just check for the macro name
        assert!(code.contains("println"));
    }
}
