use std::ops::{Deref, DerefMut};

use quote::quote;
use syn::Expr;

use crate::output::ViewWriter;

impl ViewWriter {
    pub fn begin_if<'a>(&'a mut self, cond: &'a Expr) -> ViewWriterIf<'a> {
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
        let parent = self.detach();
        ViewWriterElse::new(parent, self.capacity)
    }

    fn detach(&mut self) -> &'a mut ViewWriter {
        let parent = self.parent.take().expect("was already flushed");
        parent.flush();
        self.flush();
        let cond = self.cond;
        let body = &self.writer.tokens;
        parent.push_raw(quote! { if #cond { #body } });
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
            self.detach();
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
        self.flush();
        let tokens = &self.writer.tokens;
        self.parent.push_raw(quote! { else { #tokens } });
        // The capacity needed for an if-else is the shortest of the two.
        self.parent.capacity += self.if_capacity.min(self.writer.capacity);
    }
}
