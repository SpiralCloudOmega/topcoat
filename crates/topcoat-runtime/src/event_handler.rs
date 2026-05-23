use topcoat_view::runtime::{IntoViewParts, Unescaped, ViewPart};

use crate::Expr;

/// An event handler attribute. Emits a JavaScript closure expression into a
/// `data-topcoat-on:<event>` attribute on the element. The browser scanner
/// wraps it in `new Function('__context', …)` to obtain a real handler.
#[derive(Debug, Clone)]
pub struct EventHandler<K, V> {
    key: K,
    value: V,
}

impl<K, V> EventHandler<K, V> {
    #[inline]
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> IntoViewParts for EventHandler<K, V>
where
    K: IntoViewParts,
    V: Expr<Output = ()>,
{
    fn into_view_parts(self) -> impl Iterator<Item = ViewPart> {
        let mut js = String::new();
        self.value.to_js(&mut js);

        Unescaped::new_unchecked(" data-topcoat-on:")
            .into_view_parts()
            .chain(self.key.into_view_parts())
            .chain(Unescaped::new_unchecked("=\"").into_view_parts())
            .chain(js.into_view_parts())
            .chain(Unescaped::new_unchecked("\" ").into_view_parts())
    }
}
