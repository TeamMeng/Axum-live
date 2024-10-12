use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(socket_handler)
}

async fn socket_handler(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                println!("Got {}", text);
                let msg = format!("Hello, I got your message, you said {}", text);
                socket.send(Message::Text(msg)).await.unwrap();
            }
            Message::Close(_) => {
                println!("Close....");
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:3001";

    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);

    let app = Router::new().route("/ws", get(ws_handler));

    axum::serve(listener, app).await?;

    Ok(())
}
