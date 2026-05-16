use quote::quote;
use syn::{Expr, LitStr, Token, braced, bracketed, parenthesized, token::Paren};

use crate::ast::view::view_writer::ViewWriter;

pub enum AttributeValue {
    Expr { paren: Paren, expr: Box<Expr> },
    LitStr(LitStr),
    LitInt(LitInt),
    LitFloat(LitFloat),
    LitBool(LitBool),
    Path(Path),
    Call(ExprCall),
    Macro(ExprMacro),
}

impl AttributeValue {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        match self {
            Self::Expr { expr, .. } => writer.write_expr(expr.to_token_stream()),
            Self::LitStr(inner) => writer.write_str(&inner.value()),
            Self::LitInt(inner) => writer.write_str(inner.base10_digits()),
            Self::LitFloat(inner) => writer.write_str(inner.base10_digits()),
            Self::LitBool(inner) => writer.write_str(if inner.value { "true" } else { "false" }),
            Self::Path(inner) => writer.write_expr(inner.to_token_stream()),
            Self::Call(inner) => writer.write_expr(inner.to_token_stream()),
            Self::Macro(inner) => writer.write_expr(inner.to_token_stream()),
        }
    }
}

impl Parse for AttributeValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Paren) {
            let content;
            Ok(Self::Expr {
                paren: parenthesized!(content in input),
                expr: content.parse()?,
            })
        } else if lookahead.peek(LitStr) {
            Ok(Self::LitStr(input.parse()?))
        } else if lookahead.peek(LitInt) {
            Ok(Self::LitInt(input.parse()?))
        } else if lookahead.peek(LitFloat) {
            Ok(Self::LitFloat(input.parse()?))
        } else if lookahead.peek(LitBool) {
            Ok(Self::LitBool(input.parse()?))
        } else if lookahead.peek(Ident::peek_any) || lookahead.peek(Token![::]) {
            let path: Path = input.parse()?;
            if input.peek(Token![!]) {
                let bang_token = input.parse()?;
                let content;
                let delimiter = if input.peek(Paren) {
                    MacroDelimiter::Paren(parenthesized!(content in input))
                } else if input.peek(Bracket) {
                    MacroDelimiter::Bracket(bracketed!(content in input))
                } else if input.peek(Brace) {
                    MacroDelimiter::Brace(braced!(content in input))
                } else {
                    return Err(input.error("expected `(`, `[`, or `{` after `!`"));
                };
                Ok(Self::Macro(ExprMacro {
                    attrs: Vec::new(),
                    mac: Macro {
                        path,
                        bang_token,
                        delimiter,
                        tokens: content.parse()?,
                    },
                }))
            } else if input.peek(Paren) {
                let content;
                let paren_token = parenthesized!(content in input);
                let args = Punctuated::parse_terminated(&content)?;
                Ok(Self::Call(ExprCall {
                    attrs: Vec::new(),
                    func: Box::new(Expr::Path(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path,
                    })),
                    paren_token,
                    args,
                }))
            } else {
                Ok(Self::Path(path))
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for AttributeValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Expr { expr, .. } => quote! {{ #expr }}.to_tokens(tokens),
            Self::LitStr(inner) => inner.to_tokens(tokens),
            Self::LitInt(inner) => inner.to_tokens(tokens),
            Self::LitFloat(inner) => inner.to_tokens(tokens),
            Self::LitBool(inner) => inner.to_tokens(tokens),
            Self::Path(inner) => inner.to_tokens(tokens),
            Self::Call(inner) => inner.to_tokens(tokens),
            Self::Macro(inner) => inner.to_tokens(tokens),
        }
    }
}

#[cfg(feature = "pretty")]
impl topcoat_pretty::PrettyPrint for AttributeValue {
    fn pretty_print(&self, printer: &mut topcoat_pretty::Printer<'_>) {
        match self {
            Self::LitStr(inner) => inner.pretty_print(printer),
            Self::LitInt(inner) => syn::Lit::Int(inner.clone()).pretty_print(printer),
            Self::LitFloat(inner) => syn::Lit::Float(inner.clone()).pretty_print(printer),
            Self::LitBool(inner) => syn::Lit::Bool(inner.clone()).pretty_print(printer),
            Self::Path(inner) => inner.pretty_print(printer),
            Self::Expr { paren, expr } => {
                use topcoat_pretty::{BreakMode, Delim};
                paren.pretty_print(printer, Some(BreakMode::Inconsistent), |printer| {
                    expr.pretty_print(printer);
                });
            }
            Self::Call(inner) => {
                syn::Expr::Call(inner.clone()).pretty_print(printer);
            }
            Self::Macro(inner) => {
                syn::Expr::Macro(inner.clone()).pretty_print(printer);
            }
        }
    }
}
