use proc_macro2::TokenStream;
use quote::quote;
use syn::UnOp;

use super::Expr;

impl Expr {
    pub(super) fn expr_deref_to_tokens(unary: &syn::ExprUnary) -> syn::Result<TokenStream> {
        let UnOp::Deref(_) = unary.op else {
            return Err(syn::Error::new_spanned(
                unary.op,
                "unsupported unary operator",
            ));
        };
        let inner = Self::dispatch(&unary.expr)?;
        Ok(quote! {
            ::topcoat::interop::ExprDeref::new(#inner)
        })
    }
}
