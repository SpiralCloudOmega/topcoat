use std::{borrow::Cow, ops::Deref};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    inner: Cow<'static, str>,
}

impl Path {
    pub fn new(path: impl Into<Cow<'static, str>>) -> Self {
        Self { inner: path.into() }
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
