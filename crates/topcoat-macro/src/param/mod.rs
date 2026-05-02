use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Ident, Path, Token, Visibility,
    parse::{Parse, ParseStream},
};
use topcoat_view::ast::ParseOption;

pub struct Param {
    vis: Visibility,
    name: Ident,
    fn_name: Option<ParamFnName>,
    ty: Option<ParamType>,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            vis: input.parse()?,
            name: input.parse()?,
            fn_name: input.call(ParamFnName::parse_option)?,
            ty: input.call(ParamType::parse_option)?,
        })
    }
}

struct ParamFnName {
    _as_token: Token![as],
    name: Ident,
}

impl Parse for ParamFnName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _as_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl ParseOption for ParamFnName {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![as])
    }
}

struct ParamType {
    _colon_token: Token![:],
    path: Path,
}

impl Parse for ParamType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _colon_token: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl ParseOption for ParamType {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![:])
    }
}

impl ToTokens for Param {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let name_string = self.name.to_string();
        let fn_name = self
            .fn_name
            .as_ref()
            .map(|fn_name| &fn_name.name)
            .unwrap_or(&self.name);

        quote! {
            #vis fn #fn_name(cx: &::topcoat::context::Cx) -> &str {
                for (key, value) in ::topcoat::context::raw_path_params(cx) {
                    if key == #name_string {
                        return value;
                    }
                }
                panic!("path parameter \"{}\" was not found in request path", #name_string);
            }
        }
        .to_tokens(tokens);
    }
}
