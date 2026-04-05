use std::ops::{Deref, DerefMut};

use quote::{ToTokens, quote};
use syn::Expr;

use crate::output::ViewWriter;

impl ViewWriter {
    pub fn begin_if<'a>(&'a mut self, cond: &'a Expr) -> ViewWriterIf<'a> {
        self.flush();
        ViewWriterIf::new(self, cond)
    }
}

pub(crate) struct ViewWriterIf<'a> {
    parent: Option<&'a mut ViewWriter>,
    cond: &'a Expr,
    writer: ViewWriter,
}

impl<'a> ViewWriterIf<'a> {
    pub(super) fn new(parent: &'a mut ViewWriter, cond: &'a Expr) -> Self {
        Self {
            parent: Some(parent),
            cond,
            writer: ViewWriter::new(),
        }
    }

    pub fn begin_else(mut self) -> ViewWriterElse<'a> {
        let writer = self.flush();
        ViewWriterElse::new(writer, self.capacity)
    }

    fn flush(&mut self) -> &'a mut ViewWriter {
        let parent = self.parent.take().expect("was already flushed");
        let cond = self.cond;
        self.writer.flush();
        let body = &self.writer.tokens;
        quote! { if #cond { #body } }.to_tokens(&mut parent.tokens);
        parent
    }
}

impl Deref for ViewWriterIf<'_> {
    type Target = ViewWriter;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for ViewWriterIf<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl Drop for ViewWriterIf<'_> {
    fn drop(&mut self) {
        if self.parent.is_some() {
            self.flush();
        }
    }
}

pub(crate) struct ViewWriterElse<'a> {
    parent: &'a mut ViewWriter,
    writer: ViewWriter,
    if_capacity: usize,
}

impl<'a> ViewWriterElse<'a> {
    pub(super) fn new(parent: &'a mut ViewWriter, if_capacity: usize) -> Self {
        Self {
            parent,
            writer: ViewWriter::new(),
            if_capacity,
        }
    }
}

impl Deref for ViewWriterElse<'_> {
    type Target = ViewWriter;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for ViewWriterElse<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl Drop for ViewWriterElse<'_> {
    fn drop(&mut self) {
        self.writer.flush();
        let tokens = &self.writer.tokens;
        quote! { else { #tokens } }.to_tokens(&mut self.parent.tokens);
        // The capacity needed for an if-else is the shortest of the two.
        self.parent.capacity += self.if_capacity.min(self.capacity);
    }
}
