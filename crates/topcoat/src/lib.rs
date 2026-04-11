pub use topcoat_macro::*;

pub mod component {
    pub trait Component {
        fn render(self) -> impl Future<Output = crate::view::View> + Send;
    }
}

pub mod router {
    pub use topcoat_router::*;
}

pub mod view {
    pub use topcoat_view::runtime::*;
}
