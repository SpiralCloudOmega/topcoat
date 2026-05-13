mod component;
mod formatter;
mod fragment;
mod view;

pub use component::*;
pub use formatter::*;
pub use fragment::*;
pub use view::*;

use topcoat_core::context::Cx;

pub fn render(cx: &Cx, view: View) -> String {
    let mut buf = String::new();
    let mut f = Formatter::new(&mut buf);
    view.fmt(cx, &mut f);
    buf
}
