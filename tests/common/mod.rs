#![allow(unused)]

use axum_test::{TestResponse, TestServer};
use serde_json::json;

pub async fn shorten_url(server: &TestServer, url: &str) -> TestResponse {
    server.post("/shorten").json(&json!({"url": url})).await
}

pub async fn follow_short_code(server: &TestServer, short_code: &str) -> TestResponse {
    server.get(&format!("/{short_code}")).await
}
