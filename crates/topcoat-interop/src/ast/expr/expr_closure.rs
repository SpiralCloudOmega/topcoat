use proc_macro2::TokenStream;
use quote::quote;
use syn::{Pat, Type};

use super::Expr;

/// A closure parameter binding. The type is whatever the user wrote on the
/// closure (e.g. `|e: Event|`); `None` means the user left the parameter
/// un-annotated.
struct BoundParam<'a> {
    name: &'a syn::Ident,
    ty: Option<&'a Type>,
}

impl Expr {
    pub(super) fn expr_closure_to_tokens(
        closure: &syn::ExprClosure,
    ) -> syn::Result<TokenStream> {
        let params: Vec<BoundParam> = closure
            .inputs
            .iter()
            .map(|pat| match pat {
                Pat::Ident(pi) => Ok(BoundParam {
                    name: &pi.ident,
                    ty: None,
                }),
                Pat::Type(pt) => {
                    let Pat::Ident(pi) = &*pt.pat else {
                        return Err(syn::Error::new_spanned(
                            &pt.pat,
                            "expected a bare parameter name",
                        ));
                    };
                    Ok(BoundParam {
                        name: &pi.ident,
                        ty: Some(&pt.ty),
                    })
                }
                other => Err(syn::Error::new_spanned(
                    other,
                    "expected a bare parameter name",
                )),
            })
            .collect::<syn::Result<_>>()?;
        let bindings = params.iter().map(|p| {
            let name_ident = p.name;
            let name_str = name_ident.to_string();
            match &p.ty {
                Some(ty) => quote! {
                    let #name_ident = ::topcoat::interop::ExprParam::<#ty>::new(#name_str);
                },
                None => quote! {
                    let #name_ident = ::topcoat::interop::ExprParam::new(#name_str);
                },
            }
        });
        let refs = params.iter().map(|p| {
            let name_ident = p.name;
            quote! { &#name_ident }
        });
        // One `_` per param — gives the inherent-impl lookup enough shape
        // (the tuple's arity) to pick the right per-arity `new` while leaving
        // each `Ti` to inference.
        let arity_placeholders = params.iter().map(|_| quote! { _ });
        let body = Self::dispatch(&closure.body)?;
        Ok(quote! {
            {
                #(#bindings)*
                <::topcoat::interop::ExprClosure<( #(#arity_placeholders,)* ), _>>::new(
                    ( #(#refs,)* ),
                    #body,
                )
            }
        })
    }
}
