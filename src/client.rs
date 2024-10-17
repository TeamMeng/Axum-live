// use futures::TryStreamExt;
// use reqwest::{Client, StatusCode};
// use std::str::from_utf8;

// #[tokio::main]
// async fn main() {
//     let response = Client::new()
//         .get("http://localhost:8080/sse")
//         .send()
//         .await
//         .unwrap();

//     if response.status() == StatusCode::OK {
//         let mut stream = response.bytes_stream();
//         while let Some(chunk) = stream.try_next().await.unwrap() {
//             let chunk_string = from_utf8(&chunk).unwrap();

//             println!("{:?}", chunk_string);
//         }
//     } else {
//         println!("{:?}", response.text().await.unwrap());
//     }
// }

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio::net::TcpListener;

async fn html() -> impl IntoResponse {
    Html(
        r#"
        <script>
            fetch('http://localhost:5000/data')
            .then(response => response.json())
            .then(data => console.log(data));
        </script>
        "#,
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(html));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap()
}
