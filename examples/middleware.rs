use anyhow::Result;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8080";

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    let app = Router::new()
        .route("/", get(index_handler))
        .layer(from_fn(skip));

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> &'static str {
    "Hello World"
}

async fn skip(request: Request, next: Next) -> Response {
    match request.headers().get("Authorization") {
        Some(_) => next.run(request).await.into_response(),
        None => StatusCode::FORBIDDEN.into_response(),
    }
}
