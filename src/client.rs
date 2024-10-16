use std::str::from_utf8;

use futures::TryStreamExt;
use reqwest::{Client, StatusCode};

#[tokio::main]
async fn main() {
    let response = Client::new()
        .get("http://localhost:8080/sse")
        .send()
        .await
        .unwrap();

    if response.status() == StatusCode::OK {
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.try_next().await.unwrap() {
            let chunk_string = from_utf8(&chunk).unwrap();

            println!("{:?}", chunk_string);
        }
    } else {
        println!("{:?}", response.text().await.unwrap());
    }
}
