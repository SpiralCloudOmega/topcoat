use syn::{
    Ident, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use crate::ast::{ParseOption, view::ViewWriter};

/// A single `name=value` attribute on an [`Element`](super::Element) or
/// [`Component`](super::Component).
pub struct Attribute {
    pub name: Ident,
    pub eq: Token![=],
    pub value: AttributeValue,
}

impl Attribute {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        let name = self.name.to_string();
        writer.write_str_unescaped(&name);
        writer.write_str_unescaped("=\"");
        self.value.write(writer);
        writer.write_str_unescaped("\"");
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            // Accept Rust keywords as attribute names.
            name: Ident::parse_any(input)?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl ParseOption for Attribute {
    fn peek(input: ParseStream) -> bool {
        input.peek(Ident::peek_any) && input.peek2(Token![=])
    }
}

#[cfg(feature = "pretty")]
impl topcoat_pretty::PrettyPrint for Attribute {
    fn pretty_print(&self, printer: &mut topcoat_pretty::Printer<'_>) {
        self.name.pretty_print(printer);
        self.eq.pretty_print(printer);
        self.value.pretty_print(printer);
    }
}

/// The full list of attributes attached to a single tag.
pub struct Attributes {
    pub items: Vec<Attribute>,
}

impl Attributes {
    pub(crate) fn write(&self, writer: &mut ViewWriter) {
        for item in &self.items {
            writer.write_str_unescaped(" ");
            item.write(writer);
        }
    }

    /// Returns `true` if `self` has no attributes.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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

#[cfg(feature = "pretty")]
impl topcoat_pretty::PrettyPrint for Attributes {
    fn pretty_print(&self, printer: &mut topcoat_pretty::Printer<'_>) {
        if self.items.is_empty() {
            return;
        }
        for item in &self.items {
            printer.scan_break();
            " ".pretty_print(printer);
            item.pretty_print(printer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_valued_attribute() {
        let attr: Attribute = syn::parse_str(r#"href="/about""#).unwrap();
        assert_eq!(attr.name.to_string(), "href");
        let AttributeValue::LitStr(lit) = &attr.value else {
            panic!("expected literal value");
        };
        assert_eq!(lit.value(), "/about");
    }

    #[test]
    fn parses_expression_valued_attribute() {
        let attr: Attribute = syn::parse_str("href=(url)").unwrap();
        assert!(matches!(attr.value, AttributeValue::Expr { .. }));
    }

    #[test]
    fn parses_multiple_attributes() {
        let attrs: Attributes = syn::parse_str(r#"type="text" name="q""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert_eq!(attrs.items[0].name.to_string(), "type");
        assert_eq!(attrs.items[1].name.to_string(), "name");
    }

    #[test]
    fn empty_input_yields_empty_attributes() {
        let attrs: Attributes = syn::parse_str("").unwrap();
        assert!(attrs.is_empty());
    }

    #[test]
    fn parses_call_valued_attribute() {
        let attr: Attribute = syn::parse_str("onclick=handle()").unwrap();
        assert_eq!(attr.name.to_string(), "onclick");
        assert!(matches!(attr.value, AttributeValue::Call(_)));
    }

    #[test]
    fn parses_call_valued_attribute_with_path() {
        let attr: Attribute = syn::parse_str("onclick=handlers::click(state)").unwrap();
        assert!(matches!(attr.value, AttributeValue::Call(_)));
    }

    #[test]
    fn parses_macro_valued_attribute() {
        let attr: Attribute = syn::parse_str(r#"title=tr!("hello")"#).unwrap();
        assert_eq!(attr.name.to_string(), "title");
        assert!(matches!(attr.value, AttributeValue::Macro(_)));
    }

    #[test]
    fn parses_call_followed_by_attribute() {
        let attrs: Attributes = syn::parse_str(r#"onclick=handle() class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert_eq!(attrs.items[0].name.to_string(), "onclick");
        assert!(matches!(attrs.items[0].value, AttributeValue::Call(_)));
        assert_eq!(attrs.items[1].name.to_string(), "class");
        assert!(matches!(attrs.items[1].value, AttributeValue::LitStr(_)));
    }

    #[test]
    fn parses_macro_followed_by_attribute() {
        let attrs: Attributes = syn::parse_str(r#"title=tr!("hello") class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert!(matches!(attrs.items[0].value, AttributeValue::Macro(_)));
        assert!(matches!(attrs.items[1].value, AttributeValue::LitStr(_)));
    }

    #[test]
    fn parses_call_between_attributes() {
        let attrs: Attributes = syn::parse_str(r#"id="x" onclick=handle() class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 3);
        assert!(matches!(attrs.items[1].value, AttributeValue::Call(_)));
    }

    #[test]
    fn parses_int_valued_attribute() {
        let attr: Attribute = syn::parse_str("tabindex=0").unwrap();
        let AttributeValue::LitInt(lit) = &attr.value else {
            panic!("expected int literal");
        };
        assert_eq!(lit.base10_digits(), "0");
    }

    #[test]
    fn parses_float_valued_attribute() {
        let attr: Attribute = syn::parse_str("opacity=0.5").unwrap();
        let AttributeValue::LitFloat(lit) = &attr.value else {
            panic!("expected float literal");
        };
        assert_eq!(lit.base10_digits(), "0.5");
    }

    #[test]
    fn parses_true_valued_attribute() {
        let attr: Attribute = syn::parse_str("disabled=true").unwrap();
        let AttributeValue::LitBool(lit) = &attr.value else {
            panic!("expected bool literal");
        };
        assert!(lit.value);
    }

    #[test]
    fn parses_false_valued_attribute() {
        let attr: Attribute = syn::parse_str("disabled=false").unwrap();
        let AttributeValue::LitBool(lit) = &attr.value else {
            panic!("expected bool literal");
        };
        assert!(!lit.value);
    }

    #[test]
    fn parses_path_valued_attribute() {
        let attr: Attribute = syn::parse_str("href=url").unwrap();
        let AttributeValue::Path(path) = &attr.value else {
            panic!("expected path");
        };
        assert!(path.is_ident("url"));
    }

    #[test]
    fn parses_multi_segment_path_valued_attribute() {
        let attr: Attribute = syn::parse_str("href=routes::home").unwrap();
        let AttributeValue::Path(path) = &attr.value else {
            panic!("expected path");
        };
        assert_eq!(path.segments.len(), 2);
    }

    #[test]
    fn parses_int_followed_by_attribute() {
        let attrs: Attributes = syn::parse_str(r#"tabindex=1 class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert!(matches!(attrs.items[0].value, AttributeValue::LitInt(_)));
        assert!(matches!(attrs.items[1].value, AttributeValue::LitStr(_)));
    }

    #[test]
    fn parses_bool_followed_by_attribute() {
        let attrs: Attributes = syn::parse_str(r#"disabled=true class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert!(matches!(attrs.items[0].value, AttributeValue::LitBool(_)));
        assert!(matches!(attrs.items[1].value, AttributeValue::LitStr(_)));
    }

    #[test]
    fn parses_path_followed_by_attribute() {
        let attrs: Attributes = syn::parse_str(r#"href=url class="foo""#).unwrap();
        assert_eq!(attrs.items.len(), 2);
        assert!(matches!(attrs.items[0].value, AttributeValue::Path(_)));
        assert!(matches!(attrs.items[1].value, AttributeValue::LitStr(_)));
    }
}
