mod common;

use crate::common::{follow_short_code, shorten_url};
use axum_test::TestServer;
use oxide_server::data_store::in_memory::InMemoryDataStore;
use oxide_server::{
    router, ErrorResponse, HealthOutput, RandomShortCodeGenerator, ShortenUrlOutput,
};
use serde_json::json;
use std::sync::Arc;

async fn test_server() -> TestServer {
    let store = Arc::new(InMemoryDataStore::new());
    TestServer::new(router(Arc::new(RandomShortCodeGenerator), store))
}

#[tokio::test]
async fn test_health() {
    let server = test_server().await;
    let response: HealthOutput = server.get("/health").await.json();
    assert_eq!(response.status, "ok");
}

#[tokio::test]
async fn test_hit() {
    let server = test_server().await;
    let shorten_response = shorten_url(&server, "https://google.com").await;
    let visit_response = follow_short_code(
        &server,
        &shorten_response.json::<ShortenUrlOutput>().short_code,
    )
    .await;
    assert_eq!(visit_response.status_code(), 302);
    assert_eq!(visit_response.header("Location"), "https://google.com");
}

#[tokio::test]
async fn test_invalid() {
    let server = test_server().await;
    let shorten_response = server
        .post("/shorten")
        .json(&json!({"url": "this isn't a url"}))
        .await;
    assert_eq!(shorten_response.status_code(), 400);
    let shorten_error: ErrorResponse = shorten_response.json();
    assert_eq!(
        shorten_error.error,
        "Invalid URL: relative URL without a base"
    );
}
#[tokio::test]
async fn test_miss() {
    let server = test_server().await;
    let visit_response = follow_short_code(&server, "thisisinvalid").await;
    assert_eq!(visit_response.status_code(), 404);
    let shorten_error: ErrorResponse = visit_response.json();
    assert_eq!(shorten_error.error, "Not found");
}
