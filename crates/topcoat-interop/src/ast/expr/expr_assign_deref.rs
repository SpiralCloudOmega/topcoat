use proc_macro2::TokenStream;
use quote::quote;
use syn::UnOp;

use super::Expr;

impl Expr {
    pub(super) fn expr_assign_deref_to_tokens(
        assign: &syn::ExprAssign,
    ) -> syn::Result<TokenStream> {
        let syn::Expr::Unary(unary) = &*assign.left else {
            return Err(syn::Error::new_spanned(
                &assign.left,
                "only `*place = value` assignments are supported",
            ));
        };
        let UnOp::Deref(_) = unary.op else {
            return Err(syn::Error::new_spanned(
                unary.op,
                "only `*place = value` assignments are supported",
            ));
        };
        let place = Self::dispatch(&unary.expr)?;
        let value = Self::dispatch(&assign.right)?;
        Ok(quote! {
            ::topcoat::interop::ExprAssignDeref::new(#place, #value)
        })
    }
}
