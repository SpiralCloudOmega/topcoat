use proc_macro2::TokenStream;
use quote::quote;
use syn::Member;

use super::Expr;

impl Expr {
    pub(super) fn expr_field_to_tokens(field: &syn::ExprField) -> syn::Result<TokenStream> {
        let receiver = Self::dispatch(&field.base)?;
        let Member::Named(name) = &field.member else {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "tuple field access is not supported",
            ));
        };
        let name_str = name.to_string();
        Ok(quote! {
            ::topcoat::interop::ExprField::new(
                #receiver,
                #name_str,
                |__receiver| __receiver.#name,
            )
        })
    }
}
