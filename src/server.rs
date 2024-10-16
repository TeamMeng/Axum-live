use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use chrono::Utc;
use futures::{stream, Stream};
use std::{convert::Infallible, time::Duration};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;

fn get_data() -> String {
    format!("New data from server at : {:?}", Utc::now())
}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data(get_data()))
        .map(Ok)
        .throttle(Duration::from_secs(5));

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(5)))
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server listening on {}", addr);

    let app = Router::new().route("/sse", get(sse_handler));

    axum::serve(listener, app).await.unwrap();
}
