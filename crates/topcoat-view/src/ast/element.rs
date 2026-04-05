use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

use crate::{
    ast::{Attributes, ParseOption, node_block::NodeBlock},
    view_writer::ViewWriter,
};

pub struct Element {
    name: Ident,
    attributes: Attributes,
    body: NodeBlock,
}

impl Element {
    pub fn write(&self, writer: &mut ViewWriter) {
        writer.push_str("<");
        let name = self.name.to_string();
        writer.push_str(&name);
        self.attributes.write(writer);
        writer.push_str(">");

        self.body.write(writer);

        writer.push_str("</");
        writer.push_str(&name);
        writer.push_str(">");
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            attributes: input.parse()?,
            body: input.parse()?,
        })
    }
}

impl ParseOption for Element {
    fn peek(input: ParseStream) -> bool {
        input.peek(Ident)
    }
}
