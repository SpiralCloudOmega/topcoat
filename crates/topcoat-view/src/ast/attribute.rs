use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

use crate::{ast::ParseOption, view_writer::ViewWriter};

pub struct Attribute {
    name: Ident,
    _eq: Token![=],
    value: LitStr,
}

impl Attribute {
    pub fn write(&self, writer: &mut ViewWriter) {
        let name = self.name.to_string();
        let value = self.value.value();
        writer.push_str(&name);
        writer.push_str("=\"");
        writer.push_escaped(&value);
        writer.push_str("\"");
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ParseOption for Attribute {
    fn peek(input: ParseStream) -> bool {
        input.peek(Ident) && input.peek2(Token![=])
    }
}

pub struct Attributes {
    items: Vec<Attribute>,
}

impl Attributes {
    pub fn write(&self, writer: &mut ViewWriter) {
        for item in &self.items {
            writer.push_str(" ");
            item.write(writer);
        }
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while let Some(attribute) = input.call(Attribute::parse_option)? {
            items.push(attribute);
        }
        Ok(Self { items })
    }
}
