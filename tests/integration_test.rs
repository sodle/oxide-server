use axum_test::{TestResponse, TestServer};
use oxide_server::data_store::in_memory::InMemoryDataStore;
use oxide_server::{
    router, ErrorResponse, HealthOutput, RandomShortCodeGenerator, ShortCodeGenerator,
    ShortenUrlOutput,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

async fn test_server() -> TestServer {
    let store = Arc::new(InMemoryDataStore::new());
    TestServer::new(router(Arc::new(RandomShortCodeGenerator), store))
}

async fn shorten_url(server: &TestServer, url: &str) -> ShortenUrlOutput {
    server
        .post("/shorten")
        .json(&json!({"url": url}))
        .await
        .json()
}

async fn follow_short_code(server: &TestServer, short_code: &str) -> TestResponse {
    server.get(&format!("/{short_code}")).await
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
    let visit_response = follow_short_code(&server, &shorten_response.short_code).await;
    assert_eq!(visit_response.status_code(), 308);
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

struct ScriptedShortCodeGenerator {
    codes: Vec<String>,
    index: Mutex<usize>,
}

impl ShortCodeGenerator for ScriptedShortCodeGenerator {
    fn generate(&self) -> String {
        let mut index = self.index.lock().unwrap();

        let code: String = self.codes[*index].clone();
        *index += 1;
        code
    }
}

async fn test_scripted_server(codes: Vec<String>) -> TestServer {
    let generator = ScriptedShortCodeGenerator {
        codes,
        index: Mutex::new(0),
    };
    let store = Arc::new(InMemoryDataStore::new());
    TestServer::new(router(Arc::new(generator), store))
}

#[tokio::test]
async fn test_collision() {
    let mut codes = Vec::new();
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillnotconflict"));

    let server = test_scripted_server(codes).await;

    let first_short_code = shorten_url(&server, "https://google.com").await.short_code;
    let second_short_code = shorten_url(&server, "https://scoott.blog").await.short_code;

    assert_ne!(first_short_code, second_short_code);
}
