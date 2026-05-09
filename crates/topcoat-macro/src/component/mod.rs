use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    FnArg, ItemFn, Pat, ReturnType,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
};

pub struct ComponentAttr {}

impl Parse for ComponentAttr {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}

pub struct ComponentItem {
    item: ItemFn,
}

impl Parse for ComponentItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: ItemFn = input.parse()?;
        if item.sig.asyncness.is_none() {
            return Err(syn::Error::new(
                item.sig.fn_token.span(),
                "components must be async",
            ));
        }
        if let ReturnType::Default = &item.sig.output {
            return Err(syn::Error::new(
                item.sig.fn_token.span(),
                "components must have a return type",
            ));
        }
        for arg in &item.sig.inputs {
            match arg {
                FnArg::Receiver(r) => {
                    return Err(syn::Error::new_spanned(
                        r,
                        "component functions cannot take a `self` receiver",
                    ));
                }
                FnArg::Typed(pat_type) => match &*pat_type.pat {
                    Pat::Ident(_) => {}
                    _ => {
                        return Err(syn::Error::new_spanned(
                            pat_type,
                            "component function arguments must be identifier patterns",
                        ));
                    }
                },
            }
        }
        Ok(Self { item })
    }
}

impl ToTokens for ComponentItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut item = self.item.clone();
        let generics = item.sig.generics.clone();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        item.sig.generics.params.insert(0, parse_quote! { '__cx });
        item.sig
            .inputs
            .insert(0, parse_quote! { __cx: &'__cx ::topcoat::context::Cx });
        let vis = &item.vis;
        let ident = &item.sig.ident;
        let ReturnType::Type(_, return_ty) = &item.sig.output else {
            unreachable!("validated in Parse");
        };

        let mut fields = Vec::new();
        let mut args = Vec::new();

        for input in self.item.sig.inputs.iter() {
            let FnArg::Typed(pat_type) = input else {
                unreachable!("validated in Parse");
            };
            let Pat::Ident(pi) = &*pat_type.pat else {
                unreachable!("validated in Parse");
            };
            let ty = &pat_type.ty;
            if pi.ident == "cx" {
                args.push(quote! { cx });
            } else {
                fields.push(quote! { #pi: #ty });
                args.push(quote! { self.#pi });
            }
        }

        let body = quote! {
            #item
            #ident(cx, #(#args),*).await
        };

        quote! {
            #[allow(non_camel_case_types)]
            #vis struct #ident #impl_generics #where_clause {
                #(#vis #fields),*
            }

            impl #impl_generics ::topcoat::component::Component for #ident #ty_generics #where_clause {
                type Error = <#return_ty as ::topcoat::internal::ResultExt>::E;

                async fn render(self, cx: &::topcoat::context::Cx) -> #return_ty {
                    #body
                }
            }
        }
        .to_tokens(tokens);
    }
}
