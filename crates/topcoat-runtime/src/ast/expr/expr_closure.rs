use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use super::{BoundParam, Expr};

/// A `|p1, p2, ...| body` event-handler closure. The body must reduce to a
/// statement (`Output = ()`); the macro enforces this via the inner expression
/// nodes it accepts.
pub struct ExprClosure {
    params: Vec<BoundParam>,
    body: Box<Expr>,
}

impl ExprClosure {
    pub fn new(params: Vec<BoundParam>, body: Expr) -> Self {
        Self {
            params,
            body: Box::new(body),
        }
    }
}

impl ToTokens for ExprClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let bindings = self.params.iter().map(|p| {
            let name_ident = &p.name;
            let name_str = p.name.to_string();
            match &p.ty {
                Some(ty) => quote! {
                    let #name_ident = ::topcoat::runtime::ExprParam::<#ty>::new(#name_str);
                },
                None => quote! {
                    let #name_ident = ::topcoat::runtime::ExprParam::new(#name_str);
                },
            }
        });
        let refs = self.params.iter().map(|p| {
            let name_ident = &p.name;
            quote! { &#name_ident }
        });
        // One `_` per param — gives the inherent-impl lookup enough shape
        // (the tuple's arity) to pick the right per-arity `new` while leaving
        // each `Ti` to inference.
        let arity_placeholders = self.params.iter().map(|_| quote! { _ });
        let body = &self.body;
        quote! {
            {
                #(#bindings)*
                <::topcoat::runtime::ExprClosure<( #(#arity_placeholders,)* ), _>>::new(
                    ( #(#refs,)* ),
                    #body,
                )
            }
        }
        .to_tokens(tokens);
    }
}
