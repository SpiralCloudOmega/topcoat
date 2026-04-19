use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    ItemFn, LitStr,
    parse::{Parse, ParseStream},
};

#[expect(
    dead_code,
    reason = "parsed for syntax validation; not yet consumed by code generation"
)]
pub struct RouteAttr {
    path: Option<LitStr>,
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.peek(LitStr).then(|| input.parse()).transpose()?,
        })
    }
}

#[expect(
    dead_code,
    reason = "parsed for syntax validation; not yet consumed by code generation"
)]
pub struct RouteItem {
    item: ItemFn,
}

impl Parse for RouteItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            item: input.parse()?,
        })
    }
}

impl ToTokens for RouteItem {
    fn to_tokens(&self, _: &mut TokenStream) {}
}
