use oxide_server::{router, RandomShortCodeGenerator};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router(Arc::new(RandomShortCodeGenerator)))
        .await
        .unwrap();
}
