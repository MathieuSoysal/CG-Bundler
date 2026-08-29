//! Rewriting of crate-qualified paths inside macro invocations.
//!
//! `syn` exposes macro arguments as an opaque `TokenStream`, so the regular
//! `visit_path_mut` traversal never reaches paths written inside `println!`,
//! `vec!`, `write!` and friends. These helpers walk the token stream directly.

use proc_macro2::{Group, Ident, Punct, Spacing, TokenStream, TokenTree};

/// How a matched crate-root identifier should be rewritten.
pub(super) enum RootRewrite<'a> {
    /// Drop `<root>::`, because the items now live at the bundle root.
    Strip,
    /// Replace `<root>` with `crate`.
    Rename,
    /// Insert a module segment: `<root>::` becomes `<root>::<module>::`.
    Qualify(&'a str),
    /// Anchor at the bundle root: `<root>::` becomes `crate::<root>::`.
    PrefixCrate,
}

/// Whether `tokens` contain a `<root>::` path prefix at any nesting level.
pub(super) fn tokens_reference_root(tokens: &TokenStream, root: &str) -> bool {
    let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();

    trees.iter().enumerate().any(|(index, tree)| match tree {
        TokenTree::Group(group) => tokens_reference_root(&group.stream(), root),
        TokenTree::Ident(ident) => ident == root && is_path_sep_at(&trees, index + 1),
        TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}

/// Rewrite every `<root>::` prefix in `tokens` according to `rewrite`.
pub(super) fn rewrite_root_in_tokens(
    tokens: TokenStream,
    root: &str,
    rewrite: &RootRewrite<'_>,
) -> TokenStream {
    let trees: Vec<TokenTree> = tokens.into_iter().collect();
    let mut out: Vec<TokenTree> = Vec::with_capacity(trees.len());
    let mut index = 0;

    while index < trees.len() {
        match &trees[index] {
            TokenTree::Group(group) => {
                let mut rebuilt = Group::new(
                    group.delimiter(),
                    rewrite_root_in_tokens(group.stream(), root, rewrite),
                );
                rebuilt.set_span(group.span());
                out.push(TokenTree::Group(rebuilt));
                index += 1;
            }
            TokenTree::Ident(ident)
                if ident == root
                    && is_path_sep_at(&trees, index + 1)
                    && !continues_longer_path(&trees, index) =>
            {
                // `::<root>::..` names the crate through the extern prelude. The
                // bundle has relocated it, so the anchor has to go with it.
                if is_path_sep_at(&trees, index.wrapping_sub(2)) {
                    out.pop();
                    out.pop();
                }

                let span = ident.span();
                match rewrite {
                    RootRewrite::Strip => index += 3,
                    RootRewrite::Rename => {
                        out.push(TokenTree::Ident(Ident::new("crate", span)));
                        index += 1;
                    }
                    RootRewrite::Qualify(module) => {
                        out.push(TokenTree::Ident(ident.clone()));
                        out.extend(path_sep(span));
                        out.push(TokenTree::Ident(Ident::new(module, span)));
                        out.extend(path_sep(span));
                        index += 3;
                    }
                    RootRewrite::PrefixCrate => {
                        out.push(TokenTree::Ident(Ident::new("crate", span)));
                        out.extend(path_sep(span));
                        out.push(TokenTree::Ident(ident.clone()));
                        index += 1;
                    }
                }
            }
            other => {
                out.push(other.clone());
                index += 1;
            }
        }
    }

    out.into_iter().collect()
}

/// A freshly built `::`, spanned at `span`.
fn path_sep(span: proc_macro2::Span) -> [TokenTree; 2] {
    let mut first = Punct::new(':', Spacing::Joint);
    first.set_span(span);
    let mut second = Punct::new(':', Spacing::Alone);
    second.set_span(span);

    [TokenTree::Punct(first), TokenTree::Punct(second)]
}

/// Whether the identifier at `index` merely continues a longer path, as the
/// `root` in `foo::root::X` does.
///
/// Only a `::` that itself follows a path segment counts. A *leading* `::`, as
/// in `::root::X`, does not: that anchors the path at the extern prelude, which
/// is exactly where the crate being inlined used to live.
fn continues_longer_path(trees: &[TokenTree], index: usize) -> bool {
    if !is_path_sep_at(trees, index.wrapping_sub(2)) {
        return false;
    }

    matches!(
        trees.get(index.wrapping_sub(3)),
        Some(TokenTree::Ident(_) | TokenTree::Group(_))
    ) || matches!(
        trees.get(index.wrapping_sub(3)),
        Some(TokenTree::Punct(punct)) if punct.as_char() == '>'
    )
}

fn is_path_sep_at(trees: &[TokenTree], index: usize) -> bool {
    matches!(
        trees.get(index),
        Some(TokenTree::Punct(punct)) if punct.as_char() == ':' && punct.spacing() == Spacing::Joint
    ) && matches!(
        trees.get(index + 1),
        Some(TokenTree::Punct(punct)) if punct.as_char() == ':'
    )
}
