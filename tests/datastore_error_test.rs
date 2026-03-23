mod common;
use crate::common::{follow_short_code, shorten_url};
use async_trait::async_trait;
use axum_test::TestServer;
use metrics_exporter_prometheus::PrometheusBuilder;
use oxide_server::data_store::in_memory::InMemoryDataStore;
use oxide_server::data_store::{DataStore, DataStoreError, UrlRecord};
use oxide_server::{router, RandomShortCodeGenerator, ShortenUrlOutput};
use std::sync::Arc;

async fn test_fragile_server(allow_get: bool, allow_put: bool, allow_exists: bool) -> TestServer {
    let prometheus_handle = PrometheusBuilder::new().build_recorder().handle();
    TestServer::new(router(
        Arc::new(RandomShortCodeGenerator),
        Arc::new(FragileDataStore::new(allow_get, allow_put, allow_exists)),
        prometheus_handle,
    ))
}

struct FragileDataStore {
    wrapped_store: InMemoryDataStore,
    allow_get: bool,
    allow_put: bool,
    allow_exists: bool,
}

impl FragileDataStore {
    fn new(allow_get: bool, allow_put: bool, allow_exists: bool) -> FragileDataStore {
        FragileDataStore {
            wrapped_store: InMemoryDataStore::new(),
            allow_get,
            allow_put,
            allow_exists,
        }
    }
}

#[async_trait]
impl DataStore for FragileDataStore {
    async fn get(&self, short_code: &str) -> Result<UrlRecord, DataStoreError> {
        if self.allow_get {
            self.wrapped_store.get(short_code).await
        } else {
            Err(DataStoreError::ConnectionError)
        }
    }

    async fn put(&self, short_code: &str, url: &str) -> Result<(), DataStoreError> {
        if self.allow_put {
            self.wrapped_store.put(short_code, url).await
        } else {
            Err(DataStoreError::ConnectionError)
        }
    }

    async fn exists(&self, short_code: &str) -> Result<bool, DataStoreError> {
        if self.allow_exists {
            self.wrapped_store.exists(short_code).await
        } else {
            Err(DataStoreError::ConnectionError)
        }
    }
}

#[tokio::test]
async fn test_read_failure() {
    let server = test_fragile_server(false, true, true).await;
    let shorten_result = shorten_url(&server, "https://google.com").await;
    let visit_result = follow_short_code(
        &server,
        &shorten_result.json::<ShortenUrlOutput>().short_code,
    )
    .await;
    assert_eq!(visit_result.status_code(), 500)
}

#[tokio::test]
async fn test_write_failure() {
    let server = test_fragile_server(true, false, true).await;
    let result = shorten_url(&server, "https://google.com").await;
    assert_eq!(result.status_code(), 500)
}

#[tokio::test]
async fn test_exists_failure() {
    let server = test_fragile_server(true, true, false).await;
    let result = shorten_url(&server, "https://google.com").await;
    assert_eq!(result.status_code(), 500)
}
