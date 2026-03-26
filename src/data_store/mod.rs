use async_trait::async_trait;
use std::fmt::{Display, Formatter};

pub mod dynamodb;
pub mod in_memory;

pub struct UrlRecord {
    pub short_code: String,
    pub url: String,
}

#[derive(Debug, PartialEq)]
pub enum DataStoreError {
    NotFound,
    ConnectionError,
    DataTypeError,
}

impl Display for DataStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[async_trait]
pub trait DataStore: Send + Sync {
    async fn get(&self, short_code: &str) -> Result<UrlRecord, DataStoreError>;
    async fn put(&self, short_code: &str, url: &str) -> Result<(), DataStoreError>;
    async fn exists(&self, short_code: &str) -> Result<bool, DataStoreError>;
}
