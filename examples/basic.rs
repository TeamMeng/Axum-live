use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::SystemTime,
};

use anyhow::Result;
use axum::{
    async_trait, debug_handler,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer as _,
};

const SECRET: &[u8] = b"deabeef";

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: usize,
    user_id: usize,
    title: String,
    completed: bool,
}

#[allow(unused)]
#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    id: usize,
    name: String,
    exp: usize,
}

#[derive(Debug, Default, Clone)]
struct TodoStore {
    items: Arc<RwLock<Vec<Todo>>>,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() -> Result<()> {
    let store = TodoStore::default();

    let layer = Layer::new()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/todos", get(todos_handler).post(create_todo_handler))
        .route("/login", post(login_handler))
        .with_state(store);

    let addr = "127.0.0.1:8080";
    info!("Server listenging on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> &'static str {
    "Hello World"
}

async fn todos_handler(
    claims: Claims,
    State(store): State<TodoStore>,
) -> Result<Json<Vec<Todo>>, HttpError> {
    let user_id = claims.id;
    match store.items.read() {
        Ok(items) => Ok(Json(
            items
                .iter()
                .filter(|todo| todo.user_id == user_id)
                .cloned()
                .collect(),
        )),
        Err(_) => Err(HttpError::Internal),
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| HttpError::Auth)?;

        let key = DecodingKey::from_secret(SECRET);
        let token =
            decode(bearer.token(), &key, &Validation::default()).map_err(|_| HttpError::Auth)?;

        Ok(token.claims)
    }
}

#[debug_handler]
async fn create_todo_handler(
    claims: Claims,
    State(store): State<TodoStore>,
    Json(todo): Json<CreateTodo>,
) -> Result<StatusCode, HttpError> {
    match store.items.write() {
        Ok(mut guard) => {
            let todo = Todo {
                id: get_next_id(),
                user_id: claims.id,
                title: todo.title,
                completed: false,
            };
            guard.push(todo);
            Ok(StatusCode::CREATED)
        }
        Err(_) => Err(HttpError::Internal),
    }
}

// token eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6MSwibmFtZSI6IlRlYW0gTWVuZyIsImV4cCI6MTcyOTQ4NjYxNH0.YibD09lLtHYiKaFeXmmaxhQP2J3YnyzKvP2R46N6kOo
async fn login_handler(Json(_login): Json<LoginRequest>) -> Json<LoginResponse> {
    // skip login info validation
    let claims = Claims {
        id: 1,
        name: "Team Meng".to_string(),
        exp: get_epoch() + 14 * 24 * 60 * 60,
    };
    let key = EncodingKey::from_secret(SECRET);
    let token = encode(&Header::default(), &claims, &key).unwrap();
    Json(LoginResponse { token })
}

fn get_epoch() -> usize {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

#[derive(Debug, Serialize, Deserialize)]
enum HttpError {
    Auth,
    Internal,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpError::Auth => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            HttpError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

fn get_next_id() -> usize {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
