pub mod data_store;

use crate::data_store::{DataStore, DataStoreError};
use crate::AppError::{InternalError, InvalidUrl, NotFound};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Clone)]
struct AppState {
    generator: Arc<dyn ShortCodeGenerator>,
    data_store: Arc<dyn DataStore>,
}

pub fn router(generator: Arc<dyn ShortCodeGenerator>, data_store: Arc<dyn DataStore>) -> Router {
    let state = AppState {
        generator,
        data_store,
    };

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
            NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: String::from("Not found"),
                }),
            )
                .into_response(),
            InvalidUrl(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid URL: {error}"),
                }),
            )
                .into_response(),
            InternalError => (
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
    Url::parse(&payload.url).map_err(|e| InvalidUrl(e.to_string()))?;

    let short_code = {
        let short_code = loop {
            let candidate = state.generator.generate();
            match state.data_store.exists(&candidate).await {
                Ok(false) => break candidate,
                Ok(true) => println!("(duplicate, retry) {candidate}"),
                Err(_) => return Err(InternalError),
            }
        };
        println!("(shorten) {short_code} -> {}", payload.url);
        short_code
    };

    match state.data_store.put(&short_code, &payload.url).await {
        Ok(_) => Ok(Json::from(ShortenUrlOutput { short_code })),
        Err(_) => Err(InternalError),
    }
}

async fn code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Redirect, AppError> {
    match state.data_store.get(&code).await {
        Ok(record) => Ok(Redirect::permanent(record.url.as_str())),
        Err(DataStoreError::NotFound) => Err(NotFound),
        Err(_) => Err(InternalError),
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
