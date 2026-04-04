use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
};

use crate::ast::{parse_option::ParseOption, view::ViewWriter};

pub struct NodeExpr {
    _paren: syn::token::Paren,
    expr: syn::Expr,
}

impl NodeExpr {
    pub fn write(&self, writer: &mut ViewWriter) {
        let expr = &self.expr;
        writer.push_expr(quote! { ::topcoat::view::View::as_str(&#expr) });
    }
}

impl Parse for NodeExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _paren: parenthesized!(content in input),
            expr: content.parse()?,
        })
    }
}

impl ParseOption for NodeExpr {
    fn peek(input: ParseStream) -> bool {
        input.peek(syn::token::Paren)
    }
}
