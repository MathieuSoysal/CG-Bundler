//! Source-level minification utilities.
//!
//! The minifier operates on already-bundled Rust source code as plain text.
//! It is intentionally string-based (not AST-based) so it can be applied
//! after `prettyplease` rendering without round-tripping through `syn`.
//!
//! Correctness around string literals is the dominant concern: cooked,
//! byte, raw and raw-byte literals must all survive whitespace collapsing
//! intact. To achieve that, every literal is replaced by a placeholder
//! before whitespace processing, and restored via a single forward scan
//! that is robust against placeholder-collision attacks.
//!
//! Comments need the opposite treatment. A `//` comment runs to the end of its
//! line, so once the lines are joined it would swallow the remainder of the
//! file; a `/* */` comment survives the join but has its interior punctuation
//! rewritten, which can move its terminator. Doc comments are therefore
//! re-encoded as equivalent `#[doc = "..."]` attributes, which are immune to
//! both, and plain comments -- which carry no meaning -- are dropped.

use std::fmt::Write;
use std::iter::Peekable;
use std::str::Chars;

use regex::Regex;

const PLACEHOLDER_MARKER: &str = "__STRING_LITERAL_";
const AMP_SPACE_GUARD: &str = "__CGB_AMP_SPACE__";
const SLASH_STAR_GUARD: &str = "__CGB_SLASH_STAR__";
const LT_SPACE_GUARD: &str = "__CGB_LT_SPACE__";

/// Collapse the given source code onto a single line while preserving
/// the contents of every string and char literal verbatim.
#[must_use]
pub fn minify_code(code: &str) -> String {
    let (preprocessed, literals) = extract_literals(code);

    let collapsed = collapse_whitespace_lines(&preprocessed);

    restore_literal_placeholders(&collapsed, &literals)
}

/// Apply [`minify_code`] and additionally remove whitespace around
/// punctuation, again preserving literals exactly.
#[must_use]
pub fn aggressive_minify_code(code: &str) -> String {
    let initial = minify_code(code);
    let (without_literals, literals) = extract_literals(&initial);

    let tightened = tighten_punctuation(&without_literals);

    restore_literal_placeholders(&tightened, &literals)
}

fn collapse_whitespace_lines(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

fn tighten_punctuation(text: &str) -> String {
    // `a & &b` (bitwise and of a reference) must not collapse into the logical `&&`.
    let amp_guard = Regex::new(r"&\s+&").expect("regex is valid");
    let guarded = amp_guard.replace_all(text, AMP_SPACE_GUARD);

    // `a / *b` (division by a dereference) must not collapse into `/*`, which
    // opens a block comment and swallows the rest of the file.
    let slash_star_guard = Regex::new(r"/\s+\*").expect("regex is valid");
    let guarded = slash_star_guard.replace_all(&guarded, SLASH_STAR_GUARD);

    // `x < <T as Trait>::CONST` must not collapse into the shift operator `<<`.
    // A genuine `a << b` is already written without an inner space, so it is
    // unaffected by this guard.
    let lt_guard = Regex::new(r"<\s+<").expect("regex is valid");
    let guarded = lt_guard.replace_all(&guarded, LT_SPACE_GUARD);

    let re = Regex::new(r"\s*([=+*/%&|^<>,;:.()\[\]{}-])\s*").expect("regex is valid");

    let mut result = re.replace_all(&guarded, "$1").to_string();
    result = result
        .replace(",}", "}")
        // NB: `,)` is deliberately *not* collapsed. A trailing comma before `)`
        // is always legal, so dropping it saves a single byte -- but `(x,)` is a
        // one-element tuple and `(x)` is a parenthesised expression, so the
        // rewrite silently changes the meaning of the program.
        .replace(",]", "]")
        // Re-insert a space between `<` and a leading `-` so that a comparison
        // like `x < -1` does not collapse into `<-`, which is a misleading
        // (and historically reserved) Rust token sequence.
        .replace("<-", "< -")
        .replace(AMP_SPACE_GUARD, "& &")
        .replace(SLASH_STAR_GUARD, "/ *")
        .replace(LT_SPACE_GUARD, "< <");
    result
}

/// Replace every literal in `code` with a unique placeholder of the form
/// `__STRING_LITERAL_<n>__` and return the rewritten text together with
/// the original literals indexed by `n`.
fn extract_literals(code: &str) -> (String, Vec<String>) {
    let mut literals: Vec<String> = Vec::new();
    let mut placeholder_index = 0usize;
    let mut preprocessed = String::with_capacity(code.len());
    let mut chars = code.chars().peekable();

    while let Some(ch) = chars.next() {
        if let Some(comment) = try_consume_comment(ch, &mut chars) {
            let (bang, text) = match comment {
                Comment::Plain => continue,
                Comment::Outer(text) => ("", text),
                Comment::Inner(text) => ("!", text),
            };
            let _ = write!(
                preprocessed,
                "#{bang}[doc={PLACEHOLDER_MARKER}{placeholder_index}__]"
            );
            literals.push(escape_as_string_literal(&text));
            placeholder_index += 1;
            continue;
        }

        let checkpoint = chars.clone();
        if let Some(literal) = try_extract_literal(ch, &mut chars) {
            let _ = write!(preprocessed, "{PLACEHOLDER_MARKER}{placeholder_index}__");
            literals.push(literal);
            placeholder_index += 1;
        } else {
            chars = checkpoint;
            preprocessed.push(ch);
        }
    }

    (preprocessed, literals)
}

/// A comment recovered from the source.
enum Comment {
    /// `// ...` or `/* ... */` -- carries no meaning and is dropped.
    Plain,
    /// `/// ...` or `/** ... */` -- outer docs, re-encoded as `#[doc = "..."]`.
    Outer(String),
    /// `//! ...` or `/*! ... */` -- inner docs, re-encoded as `#![doc = "..."]`.
    Inner(String),
}

impl Comment {
    fn from_marker(doc_marker: Option<char>, text: String) -> Self {
        match doc_marker {
            Some('!') => Self::Inner(text),
            Some(_) => Self::Outer(text),
            None => Self::Plain,
        }
    }
}

/// Consume a comment starting at `first`, if there is one.
///
/// Neither comment form survives minification. A line comment runs to the end
/// of its line, so after [`collapse_whitespace_lines`] joins the lines it would
/// comment out everything that follows it. A block comment does survive the
/// join, but [`tighten_punctuation`] rewrites the punctuation *inside* it and
/// can move its `*/` terminator. Returning the comment here lets the caller
/// re-encode documentation as an attribute, which is immune to both.
fn try_consume_comment(first: char, chars: &mut Peekable<Chars<'_>>) -> Option<Comment> {
    if first != '/' {
        return None;
    }
    match chars.peek() {
        Some('/') => {
            chars.next();
            Some(consume_line_comment(chars))
        }
        Some('*') => {
            chars.next();
            Some(consume_block_comment(chars))
        }
        _ => None,
    }
}

/// Consume the remainder of a `//` comment; the leading `//` is already eaten.
fn consume_line_comment(chars: &mut Peekable<Chars<'_>>) -> Comment {
    // `///` introduces an outer doc comment and `//!` an inner one, but a
    // fourth slash makes `////...` a plain comment again.
    let mut doc_marker = None;
    match chars.peek() {
        Some('/') => {
            chars.next();
            if chars.peek() != Some(&'/') {
                doc_marker = Some('/');
            }
        }
        Some('!') => {
            chars.next();
            doc_marker = Some('!');
        }
        _ => {}
    }

    let mut text = String::new();
    for ch in chars.by_ref() {
        if ch == '\n' {
            break;
        }
        text.push(ch);
    }
    let text = text.trim_end_matches('\r').to_owned();

    Comment::from_marker(doc_marker, text)
}

/// Consume the remainder of a `/* */` comment; the leading `/*` is already
/// eaten. Rust block comments nest, so the terminator is matched by depth.
fn consume_block_comment(chars: &mut Peekable<Chars<'_>>) -> Comment {
    // `/**` introduces an outer doc block and `/*!` an inner one, but the empty
    // `/**/` is a plain comment rather than a doc block with no terminator.
    let mut doc_marker = None;
    match chars.peek() {
        Some('*') => {
            chars.next();
            if chars.peek() == Some(&'/') {
                chars.next();
                return Comment::Plain;
            }
            doc_marker = Some('*');
        }
        Some('!') => {
            chars.next();
            doc_marker = Some('!');
        }
        _ => {}
    }

    let mut depth = 1usize;
    let mut text = String::new();
    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() == Some(&'/') {
            chars.next();
            depth -= 1;
            if depth == 0 {
                break;
            }
            text.push_str("*/");
        } else if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            depth += 1;
            text.push_str("/*");
        } else {
            text.push(ch);
        }
    }

    Comment::from_marker(doc_marker, text)
}

/// Render `text` as a Rust string literal, escaping the two characters that
/// would otherwise terminate it or start an escape sequence.
fn escape_as_string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for ch in text.chars() {
        match ch {
            '\\' => literal.push_str("\\\\"),
            '"' => literal.push_str("\\\""),
            // A block doc comment may span lines; keep the output single-line.
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            _ => literal.push(ch),
        }
    }
    literal.push('"');
    literal
}

fn try_extract_literal(first: char, chars: &mut Peekable<Chars<'_>>) -> Option<String> {
    match first {
        '"' => Some(consume_quoted_string(chars, String::from('"'))),
        '\'' => extract_char_literal(chars, String::from('\'')),
        'r' => consume_raw_string(chars, String::from('r')),
        'b' => extract_byte_literal(chars),
        _ => None,
    }
}

fn extract_char_literal(chars: &mut Peekable<Chars<'_>>, prefix: String) -> Option<String> {
    let (literal, found_closing) = consume_char_like_literal(chars, prefix);
    found_closing.then_some(literal)
}

fn extract_byte_literal(chars: &mut Peekable<Chars<'_>>) -> Option<String> {
    match chars.peek()? {
        '"' => {
            chars.next();
            Some(consume_quoted_string(chars, String::from("b\"")))
        }
        '\'' => {
            chars.next();
            extract_char_literal(chars, String::from("b'"))
        }
        'r' => {
            chars.next();
            consume_raw_string(chars, String::from("br"))
        }
        _ => None,
    }
}

fn consume_quoted_string(chars: &mut Peekable<Chars<'_>>, mut literal: String) -> String {
    let mut escaped = false;
    for inner in chars.by_ref() {
        literal.push(inner);
        if escaped {
            escaped = false;
        } else if inner == '\\' {
            escaped = true;
        } else if inner == '"' {
            break;
        }
    }
    literal
}

fn consume_char_like_literal(
    chars: &mut Peekable<Chars<'_>>,
    mut literal: String,
) -> (String, bool) {
    let mut found_closing = false;
    let mut escaped = false;

    while let Some(&next) = chars.peek() {
        if !escaped && is_char_literal_boundary(next) {
            break;
        }

        literal.push(next);
        chars.next();

        if escaped {
            escaped = false;
        } else if next == '\\' {
            escaped = true;
        } else if next == '\'' {
            found_closing = true;
            break;
        }
    }

    (literal, found_closing)
}

const fn is_char_literal_boundary(ch: char) -> bool {
    matches!(ch, ' ' | ';' | '\n' | '>' | ',' | ')')
}

fn consume_raw_string(chars: &mut Peekable<Chars<'_>>, mut literal: String) -> Option<String> {
    let hash_count = peek_raw_string_opener(chars)?;

    consume_raw_string_opener(chars, &mut literal, hash_count);
    consume_raw_string_body(chars, &mut literal, hash_count);

    Some(literal)
}

fn peek_raw_string_opener(chars: &Peekable<Chars<'_>>) -> Option<usize> {
    let mut lookahead = chars.clone();
    let mut hash_count = 0usize;

    while matches!(lookahead.peek(), Some('#')) {
        lookahead.next();
        hash_count += 1;
    }

    if matches!(lookahead.next(), Some('"')) {
        Some(hash_count)
    } else {
        None
    }
}

fn consume_raw_string_opener(
    chars: &mut Peekable<Chars<'_>>,
    literal: &mut String,
    hash_count: usize,
) {
    for _ in 0..hash_count {
        chars.next();
        literal.push('#');
    }
    chars.next();
    literal.push('"');
}

fn consume_raw_string_body(
    chars: &mut Peekable<Chars<'_>>,
    literal: &mut String,
    hash_count: usize,
) {
    while let Some(ch) = chars.next() {
        literal.push(ch);
        if ch == '"' && consume_terminator_hashes(chars, literal, hash_count) {
            return;
        }
    }
}

fn consume_terminator_hashes(
    chars: &mut Peekable<Chars<'_>>,
    literal: &mut String,
    hash_count: usize,
) -> bool {
    for _ in 0..hash_count {
        if matches!(chars.peek(), Some('#')) {
            chars.next();
            literal.push('#');
        } else {
            return false;
        }
    }
    true
}

/// Replace every `__STRING_LITERAL_<n>__` placeholder in `text` with
/// `literals[n]` using a single forward scan. This is collision-safe:
/// inserted literal content is never re-scanned, so a literal that contains
/// the textual form of another placeholder will not be corrupted.
fn restore_literal_placeholders(text: &str, literals: &[String]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(PLACEHOLDER_MARKER) {
        out.push_str(&rest[..start]);
        let after = &rest[start + PLACEHOLDER_MARKER.len()..];
        if let Some(end) = after.find("__")
            && let Ok(idx) = after[..end].parse::<usize>()
            && let Some(literal) = literals.get(idx)
        {
            out.push_str(literal);
            rest = &after[end + 2..];
            continue;
        }
        out.push_str(PLACEHOLDER_MARKER);
        rest = after;
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_code_removes_newlines() {
        let code = "fn main() {\n    let x = 5;\n    println!(\"x={}\", x);\n}";
        let result = minify_code(code);
        assert!(!result.contains('\n'));
        assert!(result.contains("fn main"));
        assert!(result.contains("let x"));
    }

    #[test]
    fn test_minify_code_removes_empty_lines() {
        let code = "fn a() {}\n\n\nfn b() {}";
        let result = minify_code(code);
        assert_eq!(result, "fn a() {} fn b() {}");
    }

    #[test]
    fn test_minify_code_preserves_newline_inside_string_literal() {
        let code = "fn main() {\n    let s = \"hello\nworld\";\n    println!(\"{}\", s);\n}";
        let result = minify_code(code);
        assert!(
            result.contains("\"hello\nworld\""),
            "newline inside string literal was clobbered: {result:?}"
        );
        let outside = result.replace("\"hello\nworld\"", "");
        assert!(!outside.contains('\n'));
    }

    #[test]
    fn test_aggressive_minify_preserves_newline_inside_string_literal() {
        let code = "fn main() {\n    let s = \"hello\nworld\";\n    println!(\"{}\", s);\n}";
        let result = aggressive_minify_code(code);
        assert!(
            result.contains("\"hello\nworld\""),
            "newline inside string literal was clobbered by aggressive minify: {result:?}"
        );
    }

    #[test]
    fn test_minify_code_preserves_raw_string_literals() {
        let code =
            "fn main() {\n    let s = r#\"hello\n\"quoted\" world\"#;\n    println!(\"{}\", s);\n}";
        let result = minify_code(code);
        assert!(
            result.contains("r#\"hello\n\"quoted\" world\"#"),
            "raw string literal was corrupted: {result:?}"
        );
    }

    #[test]
    #[allow(clippy::literal_string_with_formatting_args)]
    fn test_aggressive_minify_preserves_raw_string_literals() {
        let code = "fn main() {\n    let s = br#\"hello\n\"quoted\" world\"#;\n    println!(\"{:?}\", s);\n}";
        let result = aggressive_minify_code(code);
        assert!(
            result.contains("br#\"hello\n\"quoted\" world\"#"),
            "raw byte string literal was corrupted: {result:?}"
        );
    }

    #[test]
    fn test_minify_code_preserves_string_with_spaces_and_escapes() {
        let code = "fn main() {\n    let s = \"a   b\\n\\tc\";\n}";
        let result = minify_code(code);
        assert!(result.contains("\"a   b\\n\\tc\""), "got: {result:?}");
    }

    #[test]
    fn test_minify_code_handles_placeholder_collision() {
        let code = "fn main() {\n    let a = \"danger __STRING_LITERAL_1__ inside\";\n    let b = \"second\";\n}";
        let result = minify_code(code);
        assert!(
            result.contains("\"danger __STRING_LITERAL_1__ inside\""),
            "first literal was corrupted by placeholder collision: {result:?}"
        );
        assert!(
            result.contains("\"second\""),
            "second literal missing: {result:?}"
        );
        syn::parse_file(&result).expect("minified code must be valid Rust");
    }

    #[test]
    fn test_aggressive_minify_handles_placeholder_collision() {
        let code = "fn main() {\n    let a = \"danger __STRING_LITERAL_1__ inside\";\n    let b = \"second\";\n}";
        let result = aggressive_minify_code(code);
        assert!(
            result.contains("\"danger __STRING_LITERAL_1__ inside\""),
            "first literal was corrupted by placeholder collision: {result:?}"
        );
        assert!(
            result.contains("\"second\""),
            "second literal missing: {result:?}"
        );
        syn::parse_file(&result).expect("aggressively minified code must be valid Rust");
    }

    #[test]
    fn test_aggressive_minify_code_with_escaped_string_chars() {
        let code = r#"fn f() { let s = "hello \"world\" \\path"; }"#;
        let result = aggressive_minify_code(code);
        assert!(
            result.contains(r#""hello \"world\" \\path""#),
            "escaped string content must survive minification: {result}"
        );
    }

    #[test]
    fn test_aggressive_minify_code_with_char_literals() {
        let code = "fn f() { let _a = 'a'; let _z = 'z'; }";
        let result = aggressive_minify_code(code);
        assert!(
            result.contains("'a'") && result.contains("'z'"),
            "char literals must survive minification: {result}"
        );
    }

    #[test]
    fn test_aggressive_minify_code_with_escaped_char_literal() {
        let code = r"fn f() { let _bs = '\\'; let _nl = '\n'; }";
        let _ = aggressive_minify_code(code);
    }

    #[test]
    fn test_aggressive_minify_code() {
        let snippet = r#"
trait Printable {
    fn print(&self);
}

struct Person {
    name: String,
}

impl Printable for Person {
    fn print(&self) {
        println!("Person: {}", self.name);
    }
}

impl Person {
    fn new(name: String) -> Self {
        Person { name }
    }
}

fn main() {
    let person = Person::new("Alice".to_string());
    person.print();
}
"#;
        let expected = r#"trait Printable{fn print(&self);}struct Person{name:String}impl Printable for Person{fn print(&self){println!("Person: {}",self.name);}}impl Person{fn new(name:String)->Self{Person{name}}}fn main(){let person=Person::new("Alice".to_string());person.print();}"#;
        assert_eq!(aggressive_minify_code(snippet), expected);
    }

    #[test]
    fn test_aggressive_minify_does_not_create_spurious_arrow_sequences() {
        let snippet = r#"
fn classify(x: i32) -> &'static str {
    match x {
        n if n < -10 => "very negative",
        n if n < 0 => "negative",
        0 => "zero",
        _ => "positive",
    }
}

fn filter_neg(v: &[i32]) -> Vec<i32> {
    v.iter().filter(|&&x| x < -1).cloned().collect()
}
"#;
        let result = aggressive_minify_code(snippet);
        assert!(
            !result.contains("<-"),
            "spurious '<-' sequence found in minified output: {result}"
        );
        assert!(
            result.contains("->"),
            "'->' was unexpectedly removed from minified output: {result}"
        );
        assert!(
            result.contains("=>"),
            "'=>' was unexpectedly removed from minified output: {result}"
        );
    }
}
