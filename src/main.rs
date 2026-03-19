use oxide_server::data_store::dynamodb::DynamoDbDataStore;
use oxide_server::{router, RandomShortCodeGenerator};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let store = Arc::new(DynamoDbDataStore::new().await);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let router = router(Arc::new(RandomShortCodeGenerator), store);
    axum::serve(listener, router).await.unwrap();
}
