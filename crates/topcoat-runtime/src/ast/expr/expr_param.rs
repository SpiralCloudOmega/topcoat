use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Ident;

/// A reference to a closure parameter. The macro produces this node instead of
/// [`ExprIdent`](super::ExprIdent) when the identifier is bound by an
/// enclosing closure. The enclosing [`ExprClosure`](super::ExprClosure) emits a
/// local `let <name> = ExprParam::new("<name>");` binding (carrying any user
/// type annotation as a turbofish), so each reference just clones that
/// binding.
pub struct ExprParam {
    name: Ident,
}

impl ExprParam {
    pub fn new(name: Ident) -> Self {
        Self { name }
    }
}

impl ToTokens for ExprParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        quote! { #name.clone() }.to_tokens(tokens);
    }
}
