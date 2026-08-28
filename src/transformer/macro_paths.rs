//! Rewriting of crate-qualified paths inside macro invocations.
//!
//! `syn` exposes macro arguments as an opaque `TokenStream`, so the regular
//! `visit_path_mut` traversal never reaches paths written inside `println!`,
//! `vec!`, `write!` and friends. These helpers walk the token stream directly.

use proc_macro2::{Group, Ident, Spacing, TokenStream, TokenTree};

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
    let mut out = TokenStream::new();
    let mut previous_was_colon = false;
    let mut index = 0;

    while index < trees.len() {
        match &trees[index] {
            TokenTree::Group(group) => {
                let mut rebuilt = Group::new(
                    group.delimiter(),
                    rewrite_root_in_tokens(group.stream(), root, rewrite),
                );
                rebuilt.set_span(group.span());
                out.extend([TokenTree::Group(rebuilt)]);
                previous_was_colon = false;
                index += 1;
            }
            // A leading `::` means the path is already anchored elsewhere.
            TokenTree::Ident(ident)
                if !previous_was_colon && ident == root && is_path_sep_at(&trees, index + 1) =>
            {
                let span = ident.span();
                match rewrite {
                    RootRewrite::Strip => index += 3,
                    RootRewrite::Rename => {
                        out.extend([TokenTree::Ident(Ident::new("crate", span))]);
                        index += 1;
                    }
                    RootRewrite::Qualify(module) => {
                        out.extend([
                            TokenTree::Ident(ident.clone()),
                            trees[index + 1].clone(),
                            trees[index + 2].clone(),
                            TokenTree::Ident(Ident::new(module, span)),
                            trees[index + 1].clone(),
                            trees[index + 2].clone(),
                        ]);
                        index += 3;
                    }
                    RootRewrite::PrefixCrate => {
                        out.extend([
                            TokenTree::Ident(Ident::new("crate", span)),
                            trees[index + 1].clone(),
                            trees[index + 2].clone(),
                            TokenTree::Ident(ident.clone()),
                        ]);
                        index += 1;
                    }
                }
                previous_was_colon = false;
            }
            other => {
                previous_was_colon =
                    matches!(other, TokenTree::Punct(punct) if punct.as_char() == ':');
                out.extend([other.clone()]);
                index += 1;
            }
        }
    }

    out
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
