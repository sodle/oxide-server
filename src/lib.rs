pub mod data_store;

use crate::data_store::{DataStore, DataStoreError};
use crate::AppError::{InternalError, InvalidUrl, NotFound};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use rand::distr::SampleString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tower_http::trace::TraceLayer;
use url::Url;

#[derive(Clone)]
struct AppState {
    generator: Arc<dyn ShortCodeGenerator>,
    data_store: Arc<dyn DataStore>,
    prometheus_handle: PrometheusHandle,
}

pub fn router(generator: Arc<dyn ShortCodeGenerator>, data_store: Arc<dyn DataStore>) -> Router {
    let builder = PrometheusBuilder::new();
    let prometheus_handle = builder
        .install_recorder()
        .expect("Couldn't install Prometheus recorder");

    let state = AppState {
        generator,
        data_store,
        prometheus_handle,
    };

    gauge!("up").set(1);

    Router::new()
        .route("/health", get(health))
        .route("/shorten", post(shorten))
        .route("/{code}", get(code))
        .route("/404", get(error_404))
        .route("/metrics", get(metrics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn metrics(State(state): State<AppState>) -> Response<String> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Cache-Control", "no-store")
        .header("Content-Type", "text/plain")
        .body(state.prometheus_handle.render())
        .unwrap()
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

async fn health() -> Response {
    let health = HealthOutput {
        status: String::from("ok"),
    };
    let health_json = json!(health).to_string();

    Response::builder()
        .status(200)
        .header("Cache-Control", "no-store")
        .header("Content-Type", "application/json")
        .body(Body::from(health_json))
        .unwrap()
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
    println!("(shorten) => {}", payload.url);
    match Url::parse(&payload.url) {
        Ok(_) => (),
        Err(err) => {
            counter!("invalid_url_rejected_total").increment(1);
            return Err(InvalidUrl(err.to_string()));
        }
    };

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
        Ok(_) => {
            counter!("urls_shortened_total").increment(1);
            Ok(Json::from(ShortenUrlOutput { short_code }))
        }
        Err(_) => Err(InternalError),
    }
}

async fn code(State(state): State<AppState>, Path(code): Path<String>) -> Response {
    let start = Instant::now();
    match state.data_store.get(&code).await {
        Ok(record) => match Response::builder()
            .status(StatusCode::FOUND)
            .header("Location", &record.url)
            .header("Cache-Control", "public, max-age=300")
            .body(Body::from(()))
        {
            Ok(response) => {
                println!("hit {code} => {}", record.url);
                let duration = start.elapsed();
                histogram!("url_lookup_duration_ms", "status" => "found").record(duration);
                response
            }
            Err(err) => {
                println!("error generating response: {err}");
                InternalError.into_response()
            }
        },
        Err(DataStoreError::NotFound) => {
            println!("not found {code}");
            let duration = start.elapsed();
            histogram!("url_lookup_duration_ms", "status" => "not_found").record(duration);
            counter!("url_not_found_total").increment(1);
            NotFound.into_response()
        }
        Err(err) => {
            println!("data store error {:?}", err);
            InternalError.into_response()
        }
    }
}

async fn error_404() -> Result<Json<ErrorResponse>, AppError> {
    Ok(Json::from(ErrorResponse {
        error: String::from("The short URL you requested doesn't exist."),
    }))
}
