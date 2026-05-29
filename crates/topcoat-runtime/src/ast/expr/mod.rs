mod block;
mod expr_binary;
mod expr_block;
mod expr_closure;
mod expr_field;
mod expr_index;
mod expr_lit;
mod expr_method_call;
mod expr_paren;
mod expr_path;
mod externals;
mod pat;
mod stmt;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};

use crate::ast::expr::externals::Externals;

/// The top-level `expr! { ... }` AST. A thin wrapper around `syn::Expr`; the
/// whitelist of supported shapes is enforced when lowering to tokens.
pub struct Expr {
    inner: syn::Expr,
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            inner: input.parse()?,
        })
    }
}

impl Expr {
    pub fn expr_to_tokens(&self) -> syn::Result<TokenStream> {
        let mut rust = TokenStream::new();
        let mut js = String::new();
        Self::dispatch(&self.inner, &mut rust, &mut js)?;

        // Identifiers referenced but not declared by the expression are
        // captured from the surrounding Rust scope. Their values are encoded
        // into the JavaScript source at runtime as `const` bindings, declared
        // ahead of the returned expression.
        let externals = Externals::collect(&self.inner);
        let js_externals = externals.iter().enumerate().map(|(i, ident)| {
            let prefix = if i == 0 {
                format!("(() => {{ const {ident} = ")
            } else {
                format!("; const {ident} = ")
            };
            quote! {
                __js += #prefix;
                ::topcoat::runtime::Interop::to_js(&#ident, &mut __js);
            }
        });
        let rust_externals = externals.iter().map(|ident| {
            quote! { let #ident = ::topcoat::runtime::Interop::into_surrogate(#ident); }
        });

        let js_head = if externals.is_empty() {
            quote! { __js += "(() => {"; }
        } else {
            quote! { #(#js_externals)* }
        };
        let js_tail = if externals.is_empty() {
            format!(" return {js}; }})()")
        } else {
            format!("; return {js}; }})()")
        };

        Ok(quote! {{
            let mut __js = String::new();
            #js_head
            #(#rust_externals)*
            let __rust = #rust;
            __js += #js_tail;
            ::topcoat::runtime::Expr::new(__rust, __js)
        }})
    }

    /// Lowers a single `syn::Expr` into a Rust value (`rust`) and the
    /// equivalent JavaScript source (`js`), recursing into sub-expressions.
    fn dispatch(expr: &syn::Expr, rust: &mut TokenStream, js: &mut String) -> syn::Result<()> {
        match expr {
            syn::Expr::Lit(inner) => Self::expr_lit(inner, rust, js)?,
            syn::Expr::Paren(inner) => Self::expr_paren(inner, rust, js)?,
            syn::Expr::Binary(inner) => Self::expr_binary(inner, rust, js)?,
            syn::Expr::MethodCall(inner) => Self::expr_method_call(inner, rust, js)?,
            syn::Expr::Field(inner) => Self::expr_field(inner, rust, js)?,
            syn::Expr::Index(inner) => Self::expr_index(inner, rust, js)?,
            syn::Expr::Block(inner) => Self::expr_block(inner, rust, js)?,
            syn::Expr::Closure(inner) => Self::expr_closure(inner, rust, js)?,
            syn::Expr::Path(inner) => Self::expr_path(inner, rust, js)?,
            other => return Err(syn::Error::new_spanned(other, "unsupported expression")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;

    fn expand(input: &str) -> String {
        syn::parse_str::<Expr>(input)
            .unwrap()
            .expr_to_tokens()
            .unwrap()
            .to_string()
    }

    #[test]
    fn emits_tail_after_rust_expression_without_externals() {
        let expanded = expand("1 + 2");

        assert_eq!(expanded.matches("__js +=").count(), 2, "{expanded}");
        assert!(expanded.contains("\"(() => {\""), "{expanded}");
        assert!(expanded.contains("\" return 1 + 2; })()\""), "{expanded}");
        assert!(
            expanded.find("let __rust").unwrap()
                < expanded.find("\" return 1 + 2; })()\"").unwrap(),
            "{expanded}"
        );
    }

    #[test]
    fn folds_wrapper_strings_into_single_external_capture_prefix() {
        let expanded = expand("smep + 1");

        assert_eq!(expanded.matches("__js +=").count(), 2, "{expanded}");
        assert!(expanded.contains("\"(() => {const smep = \""), "{expanded}");
        assert!(
            expanded.contains("\"; return smep + 1; })()\""),
            "{expanded}"
        );
        assert!(
            expanded.find("let __rust").unwrap()
                < expanded.find("\"; return smep + 1; })()\"").unwrap(),
            "{expanded}"
        );
    }

    #[test]
    fn folds_separator_strings_between_external_captures() {
        let expanded = expand("smep + blep");

        assert_eq!(expanded.matches("__js +=").count(), 3, "{expanded}");
        assert!(expanded.contains("\"(() => {const smep = \""), "{expanded}");
        assert!(expanded.contains("\";const blep = \""), "{expanded}");
        assert!(
            expanded.contains("\"; return smep + blep; })()\""),
            "{expanded}"
        );
        assert!(
            expanded.find("let __rust").unwrap()
                < expanded.find("\"; return smep + blep; })()\"").unwrap(),
            "{expanded}"
        );
    }
}
