mod view_writer_for_loop;
mod view_writer_if;
mod view_writer_match;

pub(crate) use view_writer_if::*;
pub(crate) use view_writer_match::*;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

#[derive(Default)]
pub(crate) struct ViewWriter {
    pub(self) tokens: TokenStream,
    static_segment: String,
    capacity: usize,
}

impl ViewWriter {
    pub fn new() -> Self {
        Self::default()
    }

    fn flush(&mut self) {
        if !self.static_segment.is_empty() {
            let static_segment = &self.static_segment;
            quote! { ::topcoat::Fragment::fmt_unescaped(#static_segment, &mut __f); }
                .to_tokens(&mut self.tokens);
            self.capacity += self.static_segment.len();
            self.static_segment.clear();
        }
    }

    pub fn write_str_unescaped(&mut self, s: &str) {
        self.static_segment.push_str(s);
    }

    pub fn write_str(&mut self, s: &str) {
        crate::runtime::Formatter::new(&mut self.static_segment).write_str(s);
    }

    pub fn write_expr_unescaped(&mut self, expr: TokenStream) {
        self.flush();
        quote! { ::topcoat::Fragment::fmt_unescaped(&#expr, &mut __f); }
            .to_tokens(&mut self.tokens);
    }

    pub fn write_expr(&mut self, expr: TokenStream) {
        self.flush();
        quote! { ::topcoat::Fragment::fmt(&#expr, &mut __f); }.to_tokens(&mut self.tokens);
    }

    pub fn write_raw(&mut self, tokens: TokenStream) {
        tokens.to_tokens(&mut self.tokens);
    }
}

impl ToTokens for ViewWriter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let static_segment = &self.static_segment;

        // Optimized path: The view has no dynamic content. We can construct it as a &'static str.
        if self.tokens.is_empty() {
            quote! { ::topcoat::View::new(#static_segment) }.to_tokens(tokens);
            return;
        }

        let buffer = &self.tokens;
        let capacity = self.capacity + static_segment.len();
        let final_segment = (!static_segment.is_empty()).then(|| {
            quote! { ::topcoat::Fragment::fmt_unescaped(#static_segment, &mut __f); }
        });
        quote! {{
            let mut __buf = ::std::string::String::with_capacity(#capacity);
            let mut __f = ::topcoat::Formatter::new(&mut __buf);
            #buffer
            #final_segment
            ::topcoat::View::new(__buf)
        }}
        .to_tokens(tokens);
    }
}
