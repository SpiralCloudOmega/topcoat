use topcoat::{component, view, view::View};

#[component]
async fn button<'a>(id: &'a str, child: View) -> View {
    view! {
        <button id=(id) class="button">(child)</button>
    }
}

#[tokio::main]
async fn main() {
    let router = topcoat::router::Router::new();
    let content = view! {
        <html>
            <head>
                <title>"hello world"</title>
            </head>
            <body id="test">
            </body>
        </html>
    };

    println!("{}", content);
}
