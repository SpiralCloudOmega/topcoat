use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

use crate::{
    ast::{Attributes, Node, ParseOption},
    output::ViewWriter,
};

pub struct Element {
    pub opening_tag: OpeningTag,
    pub children: Vec<Node>,
    pub closing_tag: ClosingTag,
}

impl Element {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        writer.push_str("<");
        let name = self.opening_tag.name.to_string();
        writer.push_str(&name);
        self.opening_tag.attributes.write(writer);
        writer.push_str(">");

        for child in &self.children {
            child.write(writer);
        }

        writer.push_str("</");
        writer.push_str(&self.closing_tag.name.to_string());
        writer.push_str(">");
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let opening_tag: OpeningTag = input.parse()?;
        let mut children = Vec::new();
        while !input.is_empty() && !ClosingTag::peek(input) {
            children.push(input.parse()?);
        }
        let closing_tag: ClosingTag = input.parse()?;
        if closing_tag.name != opening_tag.name {
            return Err(syn::Error::new(
                closing_tag.name.span(),
                format!(
                    "closing tag `{}` does not match opening tag `{}`",
                    closing_tag.name, opening_tag.name
                ),
            ));
        }
        Ok(Self {
            opening_tag,
            children,
            closing_tag,
        })
    }
}

impl ParseOption for Element {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![<])
    }
}

pub struct OpeningTag {
    pub lt: Token![<],
    pub name: Ident,
    pub attributes: Attributes,
    pub gt: Token![>],
}

impl Parse for OpeningTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            lt: input.parse()?,
            name: input.parse()?,
            attributes: input.parse()?,
            gt: input.parse()?,
        })
    }
}

pub struct ClosingTag {
    pub lt: Token![<],
    pub slash: Token![/],
    pub name: Ident,
    pub gt: Token![>],
}

impl Parse for ClosingTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            lt: input.parse()?,
            slash: input.parse()?,
            name: input.parse()?,
            gt: input.parse()?,
        })
    }
}

impl ParseOption for ClosingTag {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![<]) && input.peek2(Token![/])
    }
}
