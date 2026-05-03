use topcoat::{
    context::Cx,
    router::{page, path_param},
    view::{View, view},
};

#[path_param]
struct PostId(uuid::Uuid);

#[page]
async fn post_page(cx: &Cx) -> View {
    view! { "showing post with id: " (PostId::of(cx).as_ref().unwrap().to_string()) }
}
