use std::ops::{Deref, DerefMut};

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::Expr;

pub(crate) struct ViewWriterScope<'a> {
    writer: &'a mut ViewWriter,
    tokens: TokenStream,
    delim: Delimiter,
}

impl Deref for ViewWriterScope<'_> {
    type Target = ViewWriter;

    fn deref(&self) -> &Self::Target {
        self.writer
    }
}

impl DerefMut for ViewWriterScope<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writer
    }
}

impl Drop for ViewWriterScope<'_> {
    fn drop(&mut self) {
        self.writer
            .push_group(Group::new(self.delim, std::mem::take(&mut self.tokens)));
    }
}

#[derive(Default)]
pub(crate) struct ViewWriter {
    tokens: TokenStream,
    static_segment: String,
    static_len: usize,
}

impl ViewWriter {
    pub fn new() -> Self {
        Self::default()
    }

    fn flush(&mut self) {
        if !self.static_segment.is_empty() {
            let static_segment = &self.static_segment;
            quote! { writer.push_str(#static_segment); }.to_tokens(&mut self.tokens);
            self.static_len += self.static_segment.len();
            self.static_segment.clear();
        }
    }

    pub fn push(&mut self, ch: char) {
        self.static_segment.push(ch);
    }

    pub fn push_str(&mut self, string: &str) {
        self.static_segment.push_str(string);
    }

    pub fn push_escaped(&mut self, string: &str) {
        for c in string.chars() {
            match c {
                '&' => self.push_str("&amp;"),
                '<' => self.push_str("&lt;"),
                '>' => self.push_str("&gt;"),
                '"' => self.push_str("&quot;"),
                '\'' => self.push_str("&#x27;"),
                _ => self.push(c),
            }
        }
    }

    pub fn push_expr(&mut self, expr: TokenStream) {
        self.flush();
        quote! { writer.push_str(#expr); }.to_tokens(&mut self.tokens);
    }

    pub fn begin_if(&mut self, cond: &Expr) -> ViewWriterScope<'_> {
        self.flush();
        quote! { if #cond }.to_tokens(&mut self.tokens);
        ViewWriterScope {
            writer: self,
            tokens: TokenStream::new(),
            delim: Delimiter::Brace,
        }
    }

    fn push_group(&mut self, group: Group) {
        self.tokens.extend([TokenTree::Group(group)]);
    }
}

impl ToTokens for ViewWriter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let static_segment = &self.static_segment;

        // Optimized path: The view has no dynamic content. We can construct it as a &'static str.
        if self.tokens.is_empty() {
            quote! { ::topcoat::view::View::new(#static_segment) }.to_tokens(tokens);
            return;
        }

        let buffer = &self.tokens;
        let static_len = self.static_len + static_segment.len();
        let final_segment = (!static_segment.is_empty()).then(|| {
            quote! { writer.push_str(#static_segment); }
        });
        quote! {{
            let mut writer = ::topcoat::view::ViewWriter::with_capacity(#static_len);
            #buffer
            #final_segment
            writer.finish()
        }}
        .to_tokens(tokens);
    }
}
