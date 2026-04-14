use std::borrow::Cow;
use std::fmt;

pub struct View {
    pub(super) buf: Cow<'static, str>,
}

impl View {
    #[inline]
    pub fn new(buf: impl Into<Cow<'static, str>>) -> Self {
        Self { buf: buf.into() }
    }
}

impl fmt::Display for View {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.buf)
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for View {
    fn into_response(self) -> axum::response::Response {
        axum::response::Html(self.buf.into_owned()).into_response()
    }
}
