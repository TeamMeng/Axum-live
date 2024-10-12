use std::time::SystemTime;

use anyhow::Result;
use axum::{
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8080";

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/login", post(login))
        .route("/logup", get(get_token));

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> &'static str {
    "Hello World"
}

async fn login(Json(login): Json<LoginInfo>) -> Result<Json<Token>, StatusCode> {
    let username = &login.username;
    let password = &login.password;

    if username.is_empty() || password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let claims = Claims {
        sub: username.to_string(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            * 24
            * 60,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    );

    match token {
        Ok(token) => Ok(Json(Token { token })),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn get_token(head_map: HeaderMap) -> Result<Json<Claims>, StatusCode> {
    if let Some(value) = head_map.get("Authorization") {
        if let Ok(str) = value.to_str() {
            if str.starts_with("Bearer ") {
                let token = str.trim_start_matches("Bearer ");
                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret("secret".as_ref()),
                    &Validation::default(),
                ) {
                    Ok(token) => return Ok(Json(token.claims)),
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
