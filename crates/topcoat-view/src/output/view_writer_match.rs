use std::ops::{Deref, DerefMut};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Pat};

use crate::output::ViewWriter;

impl ViewWriter {
    pub fn begin_match<'a>(&'a mut self, expr: &'a Expr) -> ViewWriterMatch<'a> {
        self.flush();
        ViewWriterMatch::new(self, expr)
    }
}

pub(crate) struct ViewWriterMatch<'a> {
    parent: &'a mut ViewWriter,
    expr: &'a Expr,
    arms: TokenStream,
    capacity: usize,
}

impl<'a> ViewWriterMatch<'a> {
    pub(super) fn new(parent: &'a mut ViewWriter, expr: &'a Expr) -> Self {
        Self {
            parent,
            expr,
            arms: TokenStream::new(),
            capacity: 0,
        }
    }

    pub fn begin_arm(
        &mut self,
        pat: &'a Pat,
        guard: Option<&'a Expr>,
    ) -> ViewWriterMatchArm<'_, 'a> {
        ViewWriterMatchArm::new(self, pat, guard)
    }
}

impl Drop for ViewWriterMatch<'_> {
    fn drop(&mut self) {
        self.parent.flush();
        let expr = self.expr;
        let arms = &self.arms;
        self.parent.write_raw(quote! { match #expr { #arms } });
        self.parent.capacity += self.capacity;
    }
}

pub(crate) struct ViewWriterMatchArm<'m, 'a> {
    parent: &'m mut ViewWriterMatch<'a>,
    writer: ViewWriter,
    pat: &'a Pat,
    guard: Option<&'a Expr>,
}

impl<'m, 'a> ViewWriterMatchArm<'m, 'a> {
    fn new(parent: &'m mut ViewWriterMatch<'a>, pat: &'a Pat, guard: Option<&'a Expr>) -> Self {
        Self {
            parent,
            writer: ViewWriter::new(),
            pat,
            guard,
        }
    }
}

impl Deref for ViewWriterMatchArm<'_, '_> {
    type Target = ViewWriter;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for ViewWriterMatchArm<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl Drop for ViewWriterMatchArm<'_, '_> {
    fn drop(&mut self) {
        self.writer.flush();
        let tokens = &self.writer.tokens;
        let pat = self.pat;
        let arm = if let Some(guard) = self.guard {
            quote! { #pat if #guard => { #tokens } }
        } else {
            quote! { #pat => { #tokens } }
        };
        arm.to_tokens(&mut self.parent.arms);
        // The capacity needed for a match is the capacity of the shortest branch.
        if self.parent.capacity == 0 {
            self.parent.capacity = self.writer.capacity;
        } else {
            self.parent.capacity = self.parent.capacity.min(self.writer.capacity);
        }
    }
}
