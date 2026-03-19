use crate::data_store::DataStoreError::NotFound;
use crate::data_store::{DataStore, DataStoreError, UrlRecord};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct InMemoryDataStore {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryDataStore {
    pub fn new() -> InMemoryDataStore {
        InMemoryDataStore {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DataStore for InMemoryDataStore {
    async fn get(&self, short_code: &str) -> Result<UrlRecord, DataStoreError> {
        let store = self.store.lock().unwrap();
        match store.get(short_code) {
            None => Err(NotFound),
            Some(url) => Ok(UrlRecord {
                short_code: String::from(short_code),
                url: url.clone(),
            }),
        }
    }

    async fn put(&self, short_code: &str, url: &str) -> Result<(), DataStoreError> {
        let mut store = self.store.lock().unwrap();
        store.insert(String::from(short_code), String::from(url));
        Ok(())
    }

    async fn exists(&self, short_code: &str) -> Result<bool, DataStoreError> {
        let store = self.store.lock().unwrap();
        Ok(store.contains_key(short_code))
    }
}
