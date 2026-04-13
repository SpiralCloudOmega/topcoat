use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

const PORT_START: u16 = 59039;
const PORT_RANGE: u16 = 100;

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

/// Run the WebSocket broadcast server.
///
/// Accepts connections on the given listener. When any client sends `"ready"`,
/// broadcasts `"reload"` to all other connected clients.
pub async fn run(listener: TcpListener) {
    let (tx, _) = broadcast::channel::<()>(16);
    let tx = Arc::new(tx);

    loop {
        let Ok((stream, _addr)) = listener.accept().await else {
            continue;
        };

        let Ok(ws) = tokio_tungstenite::accept_async(stream).await else {
            continue;
        };

        let tx = Arc::clone(&tx);
        tokio::spawn(handle_connection(ws, tx));
    }
}

async fn handle_connection(
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    tx: Arc<broadcast::Sender<()>>,
) {
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
