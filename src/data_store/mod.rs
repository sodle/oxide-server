use async_trait::async_trait;

pub mod dynamodb;
pub mod in_memory;

pub struct UrlRecord {
    pub short_code: String,
    pub url: String,
}

pub enum DataStoreError {
    NotFound,
    ConnectionError,
    DataTypeError,
}

#[async_trait]
pub trait DataStore: Send + Sync {
    async fn get(&self, short_code: &str) -> Result<UrlRecord, DataStoreError>;
    async fn put(&self, short_code: &str, url: &str) -> Result<(), DataStoreError>;
    async fn exists(&self, short_code: &str) -> Result<bool, DataStoreError>;
}
