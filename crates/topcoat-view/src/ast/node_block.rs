use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
};

use crate::{
    ast::{Node, parse_option::ParseOption},
    output::ViewWriter,
};

pub struct NodeBlock {
    pub brace: Brace,
    pub children: Vec<Node>,
}

impl NodeBlock {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        for child in &self.children {
            child.write(writer);
        }
    }
}

impl Parse for NodeBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            brace: braced!(content in input),
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

impl ParseOption for NodeBlock {
    fn peek(input: ParseStream) -> bool {
        input.peek(Brace)
    }
}

#[cfg(feature = "pretty")]
impl crate::pretty::PrettyPrint for NodeBlock {
    fn pretty_print(&self, printer: &mut crate::pretty::Printer<'_>) {
        use crate::pretty::{BreakMode, Delim};

        self.brace
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                for (index, node) in self.children.iter().enumerate() {
                    node.pretty_print(printer);
                    if index < self.children.len() - 1 {
                        printer.scan_same_line_trivia();
                        printer.scan_force_break();
                        " ".pretty_print(printer);
                        printer.scan_trivia(true, true);
                    }
                }
            });
    }
}
