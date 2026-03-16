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

#[derive(Clone)]
struct AppState {
    store: SharedStore,
    generator: SharedGenerator,
}

type SharedStore = Arc<Mutex<HashMap<String, String>>>;
type SharedGenerator = Arc<dyn ShortCodeGenerator>;

pub fn router(generator: Arc<dyn ShortCodeGenerator>) -> Router {
    let store: SharedStore = Arc::new(Mutex::new(HashMap::new()));
    let state = AppState { store, generator };

    Router::new()
        .route("/health", get(health))
        .route("/shorten", post(shorten))
        .route("/{code}", get(code))
        .with_state(state)
}

enum AppError {
    NotFound,
    InvalidUrl(String),
    InternalError,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
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

#[derive(Serialize, Deserialize)]
pub struct HealthOutput {
    pub status: String,
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

#[derive(Serialize, Deserialize)]
pub struct ShortenUrlOutput {
    pub short_code: String,
}

pub trait ShortCodeGenerator: Send + Sync {
    fn generate(&self) -> String;
}

pub struct RandomShortCodeGenerator;
impl ShortCodeGenerator for RandomShortCodeGenerator {
    fn generate(&self) -> String {
        rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 8)
    }
}

async fn shorten(
    State(state): State<AppState>,
    Json(payload): Json<ShortenUrlInput>,
) -> Result<Json<ShortenUrlOutput>, AppError> {
    let mut store = state.store.lock().unwrap();

    Url::parse(&payload.url).map_err(|e| AppError::InvalidUrl(e.to_string()))?;

    let short_code = loop {
        let candidate = state.generator.generate();
        if !store.contains_key(&candidate) {
            break candidate;
        }
    };
    store.insert(short_code.clone(), payload.url.clone());
    println!("(shorten) {short_code} -> {}", payload.url);

    Ok(Json::from(ShortenUrlOutput { short_code }))
}

async fn code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    let store = state.store.lock().unwrap();

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

#[cfg(test)]
mod test {
    use crate::AppError::InternalError;
    use axum::response::IntoResponse;

    #[test]
    fn test_handle_internal_error() {
        let error = InternalError.into_response();
        assert_eq!(error.status(), 500);
    }
}
