//! Code minification implementation for compressing Rust code

use crate::error::Result;
use crate::syntax_tree::SyntaxTree;
use crate::traits::CodeMinifier;
use proc_macro2::{Spacing, TokenStream, TokenTree};

/// Advanced code minifier using token stream analysis
/// Inspired by cargo-minify but adapted for code compression rather than dead code elimination
pub struct WhitespaceMinifier {
    preserve_string_literals: bool,
}

impl WhitespaceMinifier {
    pub fn new() -> Self {
        Self {
            preserve_string_literals: true,
        }
    }
    
    pub fn with_string_preservation(mut self, preserve: bool) -> Self {
        self.preserve_string_literals = preserve;
        self
    }
}

impl CodeMinifier for WhitespaceMinifier {
    fn minify(&self, tree: &SyntaxTree) -> Result<String> {
        let tokens = tree.to_token_stream();
        self.compress_token_stream(&tokens)
    }
    
    fn compress_to_single_line(&self, code: &str) -> Result<String> {
        // Parse the code into a token stream for more accurate processing
        let tokens: TokenStream = code.parse().map_err(|e| {
            crate::error::ProcessingError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse Rust code: {}", e)
            ))
        })?;
        
        self.compress_token_stream(&tokens)
    }
}

impl Default for WhitespaceMinifier {
    fn default() -> Self {
        Self::new()
    }
}

impl WhitespaceMinifier {
    /// Compress a token stream into a single line with minimal whitespace
    fn compress_token_stream(&self, tokens: &TokenStream) -> Result<String> {
        let mut result = String::new();
        let mut prev_token_type = TokenType::None;
        
        for token in tokens.clone() {
            let current_token_type = self.get_token_type(&token);
            
            // Determine if we need a space between tokens
            if self.needs_separator(prev_token_type, current_token_type, &result) {
                result.push(' ');
            }
            
            // Process the token
            self.append_token(&mut result, &token)?;
            
            prev_token_type = current_token_type;
        }
        
        // Post-process to remove any remaining doc attributes
        self.remove_doc_attributes(&result)
    }
    
    /// Remove doc attributes from the compressed output using string processing
    fn remove_doc_attributes(&self, input: &str) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = input.chars().collect();
        
        while i < chars.len() {
            if chars[i] == '#' {
                // Look ahead to see if this is a doc attribute
                if let Some(end_pos) = self.find_doc_attribute_end_from_pos(&chars, i) {
                    // Skip the entire doc attribute
                    i = end_pos + 1;
                    continue;
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        
        Ok(result)
    }
    
    /// Find the end position of a doc attribute starting from a given position
    fn find_doc_attribute_end_from_pos(&self, chars: &[char], start: usize) -> Option<usize> {
        if start >= chars.len() || chars[start] != '#' {
            return None;
        }
        
        let mut i = start + 1;
        
        // Skip whitespace after #
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        
        // Look for opening bracket
        if i >= chars.len() || chars[i] != '[' {
            return None;
        }
        i += 1; // Skip '['
        
        // Skip whitespace after [
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        
        // Look for "doc"
        if i + 3 > chars.len() || chars[i..i+3] != ['d', 'o', 'c'] {
            return None;
        }
        i += 3; // Skip "doc"
        
        // Skip whitespace after "doc"
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        
        // Look for '='
        if i >= chars.len() || chars[i] != '=' {
            return None;
        }
        i += 1; // Skip '='
        
        // Now find the closing bracket, handling nested brackets and strings
        let mut bracket_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        while i < chars.len() {
            let ch = chars[i];
            
            if escape_next {
                escape_next = false;
                i += 1;
                continue;
            }
            
            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '[' if !in_string => bracket_depth += 1,
                ']' if !in_string => {
                    if bracket_depth == 0 {
                        return Some(i); // Found the closing bracket
                    }
                    bracket_depth -= 1;
                }
                _ => {}
            }
            
            i += 1;
        }
        
        None
    }
    
    /// Determine the type of a token for spacing decisions
    fn get_token_type(&self, token: &TokenTree) -> TokenType {
        match token {
            TokenTree::Group(_) => TokenType::Group,
            TokenTree::Ident(_) => TokenType::Ident,
            TokenTree::Punct(punct) => {
                match punct.as_char() {
                    '(' | '[' | '{' => TokenType::OpenDelim,
                    ')' | ']' | '}' => TokenType::CloseDelim,
                    ',' | ';' => TokenType::Separator,
                    ':' => if punct.spacing() == Spacing::Joint { TokenType::DoubleColon } else { TokenType::Colon },
                    '!' => TokenType::Bang, // Macro bang - no space before
                    '=' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '<' | '>' => TokenType::Operator,
                    '.' => TokenType::Dot,
                    _ => TokenType::Punct,
                }
            }
            TokenTree::Literal(_) => TokenType::Literal,
        }
    }
    
    /// Determine if we need a space between two tokens
    fn needs_separator(&self, prev: TokenType, current: TokenType, prev_text: &str) -> bool {
        use TokenType::*;
        
        // Never add space at the beginning
        if prev == None || prev_text.is_empty() {
            return false;
        }
        
        match (prev, current) {
            // Never need space before these
            (_, OpenDelim) | (_, Separator) | (_, CloseDelim) | (_, Dot) | (_, Bang) => false,
            
            // Never need space after these
            (OpenDelim, _) | (Dot, _) | (DoubleColon, _) | (Bang, _) => false,
            
            // Always need space between identifiers and literals
            (Ident, Ident) | (Ident, Literal) | (Literal, Ident) | (Literal, Literal) => true,
            
            // Need space around most operators
            (_, Operator) | (Operator, _) => {
                // Exception: no space for unary operators at the beginning or after certain tokens
                if current == Operator {
                    !matches!(prev, OpenDelim | Separator | Operator | Colon)
                } else {
                    true
                }
            }
            
            // Need space after separators and colons (except double colon)
            (Separator, _) | (Colon, _) => true,
            
            // Keywords need space before identifiers
            (Ident, _) if self.is_keyword_context(prev_text) => true,
            
            _ => false,
        }
    }
    
    /// Check if the previous text ends with a keyword that needs space
    fn is_keyword_context(&self, text: &str) -> bool {
        const KEYWORDS: &[&str] = &[
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", 
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", 
            "mod", "move", "mut", "pub", "ref", "return", "static", "struct", 
            "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            "async", "await", "dyn", "union"
        ];
        
        for keyword in KEYWORDS {
            if text.ends_with(keyword) {
                // Check that it's not part of a larger identifier
                let start_idx = text.len() - keyword.len();
                if start_idx == 0 {
                    return true;
                }
                if let Some(prev_char) = text.chars().nth(start_idx - 1) {
                    if !prev_char.is_alphanumeric() && prev_char != '_' {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Append a token to the result string
    fn append_token(&self, result: &mut String, token: &TokenTree) -> Result<()> {
        match token {
            TokenTree::Group(group) => {
                // Add the opening delimiter
                let open_delim = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => "(",
                    proc_macro2::Delimiter::Brace => "{",
                    proc_macro2::Delimiter::Bracket => "[",
                    proc_macro2::Delimiter::None => "",
                };
                result.push_str(open_delim);
                
                // Recursively compress the group's contents
                let inner = self.compress_token_stream(&group.stream())?;
                result.push_str(&inner);
                
                // Add the closing delimiter
                let close_delim = match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ")",
                    proc_macro2::Delimiter::Brace => "}",
                    proc_macro2::Delimiter::Bracket => "]",
                    proc_macro2::Delimiter::None => "",
                };
                result.push_str(close_delim);
            }
            TokenTree::Ident(ident) => {
                result.push_str(&ident.to_string());
            }
            TokenTree::Punct(punct) => {
                result.push(punct.as_char());
            }
            TokenTree::Literal(literal) => {
                let lit_str = literal.to_string();
                if self.preserve_string_literals {
                    // Keep string literals as-is to preserve formatting
                    result.push_str(&lit_str);
                } else {
                    // For now, keep literals as-is since modifying them could break semantics
                    result.push_str(&lit_str);
                }
            }
        }
        Ok(())
    }
}

/// Token types for spacing decisions
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    None,
    Group,
    Ident,
    Literal,
    OpenDelim,
    CloseDelim,
    Separator,    // , ;
    Colon,        // :
    DoubleColon,  // ::
    Bang,         // ! (macro call)
    Operator,     // = + - * / % & | ^ < >
    Dot,          // .
    Punct,        // Other punctuation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SynCodeParser;
    use crate::traits::CodeParser;

    #[test]
    fn test_minify_simple_function() -> Result<()> {
        let parser = SynCodeParser::new();
        let minifier = WhitespaceMinifier::new();
        
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        
        let tree = parser.parse(code)?;
        let minified = minifier.minify(&tree)?;
        
        // Should be compressed but still valid
        assert!(!minified.contains('\n'));
        assert!(minified.contains("fn main"));
        assert!(minified.contains("println!"));
        
        Ok(())
    }
    
    #[test]
    fn test_compress_to_single_line() -> Result<()> {
        let minifier = WhitespaceMinifier::new();
        
        let code = r#"
            fn test() {
                let x = 42;
                return x;
            }
        "#;
        
        let result = minifier.compress_to_single_line(code)?;
        
        assert!(!result.contains('\n'));
        assert!(result.contains("fn test"));
        assert!(result.contains("let x"));
        assert!(result.contains("42"));
        
        Ok(())
    }
    
    #[test]
    fn test_preserve_string_literals() -> Result<()> {
        let minifier = WhitespaceMinifier::new();
        
        let code = r#"println!("Hello,    world!");"#;
        let result = minifier.compress_to_single_line(code)?;
        
        // String content should be preserved exactly
        assert!(result.contains("Hello,    world!"));
        
        Ok(())
    }
    
    #[test]
    fn test_token_spacing() -> Result<()> {
        let minifier = WhitespaceMinifier::new();
        
        let code = "fn test ( ) { let x = 42 ; return x ; }";
        let result = minifier.compress_to_single_line(code)?;
        
        // Should have proper spacing around identifiers and operators
        assert!(result.contains("fn test"));
        assert!(result.contains("let x"));
        assert!(result.contains("= 42"));
        
        Ok(())
    }
    
    #[test]
    fn test_operator_spacing() -> Result<()> {
        let minifier = WhitespaceMinifier::new();
        
        let code = "let x = a + b * c;";
        let result = minifier.compress_to_single_line(code)?;
        
        // Should have proper spacing around operators
        assert!(result.contains("= a"));
        assert!(result.contains("+ b"));
        assert!(result.contains("* c"));
        
        Ok(())
    }
    
    #[test]
    fn test_keyword_spacing() -> Result<()> {
        let minifier = WhitespaceMinifier::new();
        
        let code = "if true { return 42; }";
        let result = minifier.compress_to_single_line(code)?;
        
        // Should have proper spacing after keywords
        assert!(result.contains("if true"));
        assert!(result.contains("return 42"));
        
        Ok(())
    }
}
