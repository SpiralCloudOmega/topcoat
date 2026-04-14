use std::ops::{Deref, DerefMut};

use quote::quote;
use syn::{Expr, Pat};

use crate::output::ViewWriter;

impl ViewWriter {
    pub fn begin_for_loop<'a>(&'a mut self, pat: &'a Pat, expr: &'a Expr) -> ViewWriterForLoop<'a> {
        self.flush();
        ViewWriterForLoop::new(self, pat, expr)
    }
}

pub(crate) struct ViewWriterForLoop<'a> {
    parent: &'a mut ViewWriter,
    writer: ViewWriter,
    pat: &'a Pat,
    expr: &'a Expr,
}

impl<'a> ViewWriterForLoop<'a> {
    pub(super) fn new(parent: &'a mut ViewWriter, pat: &'a Pat, expr: &'a Expr) -> Self {
        Self {
            parent,
            writer: ViewWriter::new(),
            pat,
            expr,
        }
    }
}

impl Deref for ViewWriterForLoop<'_> {
    type Target = ViewWriter;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for ViewWriterForLoop<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl Drop for ViewWriterForLoop<'_> {
    fn drop(&mut self) {
        self.writer.flush();
        let tokens = &self.writer.tokens;
        let pat = self.pat;
        let expr = self.expr;
        self.parent
            .write_raw(quote! { for #pat in #expr { #tokens } });
    }
}
