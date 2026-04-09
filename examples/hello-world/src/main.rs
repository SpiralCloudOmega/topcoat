use topcoat::{component, view, view::View};

#[component]
async fn button(child: View) -> View {
    view! {
        <button class="button">(child)</button>
    }
}

#[tokio::main]
async fn main() {
    let content = view! {
        <html>
            <head>
                <title>"hello world"</title>
            </head>
            <body id="test">
                [button]
                    "click me"
                [/button]
            </body>
        </html>
    };

    println!("{}", content);
}
