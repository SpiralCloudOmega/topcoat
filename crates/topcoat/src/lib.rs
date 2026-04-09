pub use topcoat_macro::*;

pub mod view {
    pub use topcoat_view::runtime::*;
}

pub mod component {
    pub trait Component {
        fn render(self) -> impl Future<Output = crate::view::View> + Send;
    }
}
