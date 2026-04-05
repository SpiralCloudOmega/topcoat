use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

use crate::{ast::node::Node, view_writer::ViewWriter};

pub struct View {
    nodes: Vec<Node>,
}

impl Parse for View {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            nodes: {
                let mut children = Vec::new();
                while !input.is_empty() {
                    children.push(input.parse()?)
                }
                children
            },
        })
    }
}

impl ToTokens for View {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut writer = ViewWriter::new();
        for node in &self.nodes {
            node.write(&mut writer);
        }
        writer.to_tokens(tokens);
    }
}
