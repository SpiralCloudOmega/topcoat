use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    ExprIf, Token,
    parse::{Parse, ParseStream},
};

use crate::ast::{NodeBlock, ViewWriter, parse_option::ParseOption};

pub struct NodeIf {
    if_token: Token![if],
    cond: syn::Expr,
    then_branch: NodeBlock,
    else_branch: Option<NodeElse>,
}

impl NodeIf {
    pub fn write(&self, writer: &mut ViewWriter) {}
}

impl Parse for NodeIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            if_token: input.parse()?,
            cond: input.parse()?,
            then_branch: input.parse()?,
            else_branch: input.call(NodeElse::parse_option)?,
        })
    }
}

pub enum NodeElse {
    ElseIf {
        else_token: Token![else],
        node_if: Box<NodeIf>,
    },
    Else {
        else_token: Token![else],
        then_branch: NodeBlock,
    },
}

impl Parse for NodeElse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let else_token: Token![else] = input.parse()?;
        if input.peek(Token![if]) {
            Ok(Self::ElseIf {
                else_token,
                node_if: input.parse()?,
            })
        } else {
            Ok(Self::Else {
                else_token,
                then_branch: input.parse()?,
            })
        }
    }
}

impl ParseOption for NodeElse {
    fn peek(input: ParseStream) -> bool {
        input.peek(Token![else])
    }
}
