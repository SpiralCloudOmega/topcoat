use proc_macro2::TokenStream;
use quote::quote;

use super::Expr;

impl Expr {
    pub(super) fn expr_ident_to_tokens(path: &syn::ExprPath) -> syn::Result<TokenStream> {
        let Some(ident) = path.path.get_ident() else {
            return Err(syn::Error::new_spanned(path, "expected a bare identifier"));
        };
        Ok(quote! {
            ::topcoat::interop::IntoExpr::into_expr(#ident)
        })
    }
}
