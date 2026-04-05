use syn::{
    braced,
    parse::{Parse, ParseStream},
};

use crate::{ast::Node, view_writer::ViewWriter};

pub struct NodeBlock {
    _brace: syn::token::Brace,
    children: Vec<Node>,
}

impl NodeBlock {
    pub fn write(&self, writer: &mut ViewWriter) {
        for child in &self.children {
            child.write(writer);
        }
    }
}

impl Parse for NodeBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _brace: braced!(content in input),
            children: {
                let mut children = Vec::new();
                while !content.is_empty() {
                    children.push(content.parse()?)
                }
                children
            },
        })
    }
}
