use proc_macro2::TokenStream;
use quote::quote;

use super::Expr;

impl Expr {
    pub(super) fn expr_method_call_to_tokens(
        mc: &syn::ExprMethodCall,
    ) -> syn::Result<TokenStream> {
        if mc.turbofish.is_some() {
            return Err(syn::Error::new_spanned(
                &mc.turbofish,
                "turbofish on method calls is not supported",
            ));
        }
        if !mc.args.is_empty() {
            return Err(syn::Error::new_spanned(
                &mc.args,
                "method arguments are not supported",
            ));
        }
        let receiver = Self::dispatch(&mc.receiver)?;
        let method = &mc.method;
        let method_str = method.to_string();
        Ok(quote! {
            ::topcoat::interop::ExprMethodCall::new(
                #receiver,
                #method_str,
                |__receiver| __receiver.#method(),
            )
        })
    }
}
