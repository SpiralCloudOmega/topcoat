use topcoat::{
    context::Cx,
    router::{page, param, segment},
    view::{View, view},
};

segment!(kind = Param);
param!(id);

#[page]
async fn post_page(cx: &Cx) -> View {
    view! { "showing post with id: " (id(cx)) }
}
