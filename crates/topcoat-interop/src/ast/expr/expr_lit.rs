use proc_macro2::TokenStream;
use quote::quote;

use super::Expr;

impl Expr {
    pub(super) fn expr_lit_to_tokens(lit: &syn::ExprLit) -> syn::Result<TokenStream> {
        let inner = &lit.lit;
        Ok(quote! { ::topcoat::interop::ExprValue::new(#inner) })
    }
}
