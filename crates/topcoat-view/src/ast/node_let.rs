use quote::quote;
use syn::{
    ExprLet, Token,
    parse::{Parse, ParseStream},
};

use crate::{ast::parse_option::ParseOption, output::ViewWriter};

pub struct NodeLet {
    pub expr_let: ExprLet,
    pub semi_token: Token![;],
}

impl NodeLet {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        let expr_let = &self.expr_let;
        writer.write_raw(quote! { #expr_let; });
    }
}

impl Parse for NodeLet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr_let: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

impl ParseOption for NodeLet {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![let])
    }
}

#[cfg(feature = "pretty")]
impl crate::pretty::PrettyPrint for NodeLet {
    fn pretty_print(&self, printer: &mut crate::pretty::Printer<'_>) {
        self.expr_let.pretty_print(printer);
        self.semi_token.pretty_print(printer);
    }
}
