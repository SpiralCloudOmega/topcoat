use std::{fmt::Display, sync::atomic::Ordering};

use proc_macro2::Span;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Token, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
};

use crate::{
    ast::{Attributes, Node, ParseOption},
    output::ViewWriter,
};

// Optimize for the common case (normal elements).
#[allow(clippy::large_enum_variant)]
pub enum Element {
    Normal {
        opening_tag: OpeningTag,
        children: Vec<Node>,
        closing_tag: ClosingTag,
    },
    Void {
        tag: OpeningTag,
    },
}

impl Element {
    pub fn name(&self) -> &ElementName {
        match self {
            Self::Normal { opening_tag, .. } => &opening_tag.name,
            Self::Void { tag } => &tag.name,
        }
    }

    pub fn attributes(&self) -> &Attributes {
        match self {
            Self::Normal { opening_tag, .. } => &opening_tag.attributes,
            Self::Void { tag } => &tag.attributes,
        }
    }

    pub fn children(&self) -> &[Node] {
        match self {
            Self::Normal { children, .. } => children,
            Self::Void { .. } => &[],
        }
    }

    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        match self {
            Self::Normal {
                opening_tag,
                children,
                ..
            } => {
                // For expression attribute names, we only want to evaluate the expression once and
                // then store it in a variable.
                let name_expr = opening_tag.name.expr();
                static AUTO_INCREMENT: std::sync::atomic::AtomicU32 =
                    std::sync::atomic::AtomicU32::new(0);
                let increment = AUTO_INCREMENT.fetch_add(1, Ordering::Relaxed);
                let name_ident = name_expr.map(|_| {
                    Ident::new(&format!("__element_name_{}", increment), Span::call_site())
                });

                writer.push_raw(quote! {});

                writer.push_str("<");
                match (name_ident.as_ref(), name_expr) {
                    (Some(ident), Some(expr)) => {
                        writer.push_raw(quote! { let #ident = &#expr;  });
                        writer.push_expr(quote! { #ident });
                    }
                    _ => opening_tag.name.write(writer),
                }
                opening_tag.attributes.write(writer);
                writer.push_str(">");

                for child in children {
                    child.write(writer);
                }

                writer.push_str("</");
                match name_ident {
                    Some(ident) => writer.push_expr(quote! { #ident }),
                    _ => opening_tag.name.write(writer),
                }
                writer.push_str(">");
            }
            Self::Void { tag } => {
                writer.push_str("<");
                tag.name.write(writer);
                tag.attributes.write(writer);
                writer.push_str(">");
            }
        }
    }
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let opening_tag: OpeningTag = input.parse()?;

        if opening_tag.name.is_void_element() {
            return Ok(Self::Void { tag: opening_tag });
        }

        let mut children = Vec::new();
        while !input.is_empty() && !ClosingTag::peek(input) {
            children.push(input.parse()?);
        }

        if input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                format!("missing closing tag for opening tag `{}`", opening_tag.name),
            ));
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
        Ok(Self::Normal {
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

#[derive(PartialEq, Eq)]
pub enum ElementName {
    Ident(Ident),
    LitStr(LitStr),
    Expr { paren: Paren, expr: Expr },
}

impl ElementName {
    pub fn string_name(&self) -> Option<String> {
        match self {
            Self::Ident(inner) => Some(inner.to_string()),
            Self::LitStr(inner) => Some(inner.value()),
            Self::Expr { .. } => None,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Ident(inner) => inner.span(),
            Self::LitStr(inner) => inner.span(),
            Self::Expr { paren, .. } => paren.span.span(),
        }
    }

    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        match self {
            Self::Ident(inner) => writer.push_str(&inner.to_string()),
            Self::LitStr(inner) => writer.push_str(&inner.value()),
            Self::Expr { expr, .. } => writer.push_expr(quote! { #expr }),
        }
    }

    pub fn is_void_element(&self) -> bool {
        const VOID_ELEMENTS: &[&str] = &[
            "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
            "track", "wbr",
        ];

        match self {
            Self::Ident(inner) => {
                let name = inner.to_string();
                VOID_ELEMENTS.iter().any(|v| *v == name)
            }
            _ => false,
        }
    }

    pub fn expr(&self) -> Option<&Expr> {
        match self {
            Self::Expr { expr, .. } => Some(expr),
            _ => None,
        }
    }
}

impl Display for ElementName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(inner) => inner.fmt(f),
            Self::LitStr(inner) => inner.value().fmt(f),
            Self::Expr { .. } => f.write_str("<expr>"),
        }
    }
}

impl Parse for ElementName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if lookahead.peek(LitStr) {
            Ok(Self::LitStr(input.parse()?))
        } else if lookahead.peek(Paren) {
            let content;
            Ok(Self::Expr {
                paren: parenthesized!(content in input),
                expr: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct OpeningTag {
    pub lt: Token![<],
    pub name: ElementName,
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
    pub name: ElementName,
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
