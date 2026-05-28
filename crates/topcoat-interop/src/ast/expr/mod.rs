mod expr_assign_deref;
mod expr_closure;
mod expr_deref;
mod expr_field;
mod expr_ident;
mod expr_lit;
mod expr_method_call;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};

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
        Self::dispatch(&self.inner)
    }

    fn dispatch(expr: &syn::Expr) -> syn::Result<TokenStream> {
        match expr {
            syn::Expr::Assign(assign) => Self::expr_assign_deref_to_tokens(assign),
            syn::Expr::Closure(closure) => Self::expr_closure_to_tokens(closure),
            syn::Expr::Field(field) => Self::expr_field_to_tokens(field),
            syn::Expr::Lit(lit) => Self::expr_lit_to_tokens(lit),
            syn::Expr::MethodCall(mc) => Self::expr_method_call_to_tokens(mc),
            syn::Expr::Paren(paren) => Self::dispatch(&paren.expr),
            syn::Expr::Path(path) => Self::expr_ident_to_tokens(path),
            syn::Expr::Unary(unary) => Self::expr_deref_to_tokens(unary),
            other => Err(syn::Error::new_spanned(other, "unsupported expression")),
        }
    }
}
