use axum_test::TestServer;
use oxide_server::{
    router, ErrorResponse, HealthOutput, RandomShortCodeGenerator, ShortCodeGenerator,
    ShortenUrlOutput,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

fn test_server() -> TestServer {
    TestServer::new(router(Arc::new(RandomShortCodeGenerator)))
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

fn test_scripted_server(codes: Vec<String>) -> TestServer {
    let generator = ScriptedShortCodeGenerator {
        codes,
        index: Mutex::new(0),
    };
    TestServer::new(router(Arc::new(generator)))
}

#[tokio::test]
async fn test_collision() {
    let mut codes = Vec::new();
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillnotconflict"));

    let server = test_scripted_server(codes);

    let response = server
        .post("/shorten")
        .json(&json!({"url": "https://google.com"}))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: ShortenUrlOutput = response.json();
    let first_short_code = body.short_code;

    let response = server
        .post("/shorten")
        .json(&json!({"url": "https://scoott.blog"}))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: ShortenUrlOutput = response.json();
    let second_short_code = body.short_code;

    assert_ne!(first_short_code, second_short_code);
}
