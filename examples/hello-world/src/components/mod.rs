use topcoat::{component, view, view::View};

#[component]
async fn button(cx: Cx<'_>, id: &str, child: View) -> View {
    view! { <button id=(id) class="button">(child)</button> }
}
