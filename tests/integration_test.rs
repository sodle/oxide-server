use axum_test::TestServer;
use oxide_server::{router, ErrorResponse, HealthOutput, ShortenUrlOutput};
use serde_json::json;

fn test_server() -> TestServer {
    TestServer::new(router())
}

#[tokio::test]
async fn test_health() {
    let server = test_server();
    let response: HealthOutput = server.get("/health").await.json();
    assert_eq!(response.status, "ok");
}

#[tokio::test]
async fn test_hit() {
    let server = test_server();
    let shorten_response: ShortenUrlOutput = server
        .post("/shorten")
        .json(&json!({"url": "https://google.com"}))
        .await
        .json();
    let visit_response = server
        .get(&format!("/{}", shorten_response.short_code))
        .await;
    assert_eq!(visit_response.status_code(), 308);
    assert_eq!(visit_response.header("Location"), "https://google.com");
}

#[tokio::test]
async fn test_invalid() {
    let server = test_server();
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
    let server = test_server();
    let visit_response = server.get("/nothings").await;
    assert_eq!(visit_response.status_code(), 404);
    let shorten_error: ErrorResponse = visit_response.json();
    assert_eq!(shorten_error.error, "Not found");
}
