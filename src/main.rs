use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

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

enum AppError {
    NotFound,
    InvalidUrl(String),
    InternalError,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: String::from("Not found"),
                }),
            )
                .into_response(),
            AppError::InvalidUrl(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid URL: {error}"),
                }),
            )
                .into_response(),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: String::from("Internal error"),
                }),
            )
                .into_response(),
        }
    }
}

#[derive(Serialize)]
struct HealthOutput {
    status: String,
}

async fn health() -> Result<Json<HealthOutput>, StatusCode> {
    Ok(Json::from(HealthOutput {
        status: String::from("ok"),
    }))
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
) -> Result<Json<ShortenUrlOutput>, AppError> {
    let mut store = match store.lock() {
        Ok(store) => store,
        Err(err) => {
            println!("Could not unlock mutex: {err}");
            return Err(AppError::InternalError);
        }
    };

    match Url::parse(&*payload.url) {
        Ok(_) => {}
        Err(error) => return Err(AppError::InvalidUrl(error.to_string())),
    }

    let short_code = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 8);
    store.insert(short_code.clone(), payload.url.clone());
    println!("(shorten) {short_code} -> {}", payload.url);

    Ok(Json::from(ShortenUrlOutput { short_code }))
}

async fn code(
    State(store): State<SharedStore>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    let store = match store.lock() {
        Ok(store) => store,
        Err(err) => {
            println!("Could not unlock mutex: {err}");
            return Err(AppError::InternalError);
        }
    };

    match store.get(&code) {
        None => {
            println!("{code} -> (miss)");
            Err(AppError::NotFound)
        }
        Some(url) => {
            println!("{code} -> {url} (hit)");
            Ok(Redirect::permanent(url))
        }
    }
}
