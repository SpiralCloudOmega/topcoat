extern crate self as topcoat;

pub mod dev;
mod serve;

pub mod component {
    pub use topcoat_macro::component;

    pub trait Component {
        fn render(self) -> impl Future<Output = crate::view::View> + Send;
    }
}

pub mod router {
    pub use topcoat_macro::{layout, page, route};

    pub use topcoat_router::*;
}

pub mod view {
    pub use topcoat_macro::view;

    pub use topcoat_view::runtime::*;
}

pub use serve::serve;
