use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedStore = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let store: SharedStore = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/shorten", post(shorten))
        .route("/{code}", get(code))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct ShortenUrlInput {
    url: String,
}

#[derive(Serialize)]
struct ShortenUrlOutput {
    short_code: String,
}

async fn shorten(
    State(store): State<SharedStore>,
    Json(payload): Json<ShortenUrlInput>,
) -> Result<Json<ShortenUrlOutput>, StatusCode> {
    let mut store = match store.lock() {
        Ok(store) => store,
        Err(err) => {
            println!("Could not unlock mutex: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let short_code = format!("{}", store.len() + 1);
    store.insert(short_code.clone(), payload.url.clone());
    println!("(shorten) {short_code} -> {}", payload.url);

    Ok(Json::from(ShortenUrlOutput { short_code }))
}

async fn code(
    State(store): State<SharedStore>,
    Path(code): Path<String>,
) -> Result<Redirect, StatusCode> {
    let store = match store.lock() {
        Ok(store) => store,
        Err(err) => {
            println!("Could not unlock mutex: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match store.get(&code) {
        None => {
            println!("{code} -> (miss)");
            Err(StatusCode::NOT_FOUND)
        }
        Some(url) => {
            println!("{code} -> {url} (hit)");
            Ok(Redirect::permanent(url))
        }
    }
}
