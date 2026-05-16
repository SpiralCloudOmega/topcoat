use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

const PORT_START: u16 = 59039;
const PORT_RANGE: u16 = 100;

const DEV_JS: &str = include_str!("dev.js");

/// Bind the dev server to a stable port, starting at [`PORT_START`] and
/// incrementing if occupied.
pub async fn bind() -> TcpListener {
    for port in PORT_START..=PORT_START.saturating_add(PORT_RANGE) {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
            return listener;
        }
    }
    panic!(
        "failed to bind dev server port ({PORT_START}–{})",
        PORT_START + PORT_RANGE
    );
}

/// Run the dev server.
///
/// Serves `/dev.js` (the client reload script) and `/ws` (a WebSocket endpoint).
/// When any WS client sends `"ready"`, broadcasts `"reload"` to all other
/// connected clients.
pub async fn run(listener: TcpListener) {
    let (tx, _) = broadcast::channel::<()>(16);
    let tx = Arc::new(tx);

    let app = Router::new()
        .route("/dev.js", get(serve_dev_js))
        .route("/ws", get(ws_handler))
        .with_state(tx);

    let _ = axum::serve(listener, app).await;
}

async fn serve_dev_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        DEV_JS,
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<Arc<broadcast::Sender<()>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(ws: WebSocket, tx: Arc<broadcast::Sender<()>>) {
    let (mut sink, mut stream) = ws.split();
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            msg = stream.next() => {
                let Some(Ok(msg)) = msg else { break };

                if let Message::Text(text) = msg
                    && text == "ready" {
                        let _ = tx.send(());
                    }
            }
            Ok(()) = rx.recv() => {
                if sink.send(Message::Text("reload".into())).await.is_err() {
                    break;
                }
            }
        }
    }
}
