use crate::data_store::DataStoreError::{ConnectionError, DataTypeError, NotFound};
use crate::data_store::{DataStore, DataStoreError, UrlRecord};
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::error::DisplayErrorContext;
use aws_sdk_dynamodb::operation::get_item::GetItemOutput;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::types::AttributeValue::S;
use aws_sdk_dynamodb::Client;
use dotenv::var;
use std::collections::HashMap;

pub struct DynamoDbDataStore {
    client: Client,
    table_name: String,
}

impl DynamoDbDataStore {
    pub async fn new() -> DynamoDbDataStore {
        let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;
        DynamoDbDataStore {
            client: Client::new(&config),
            table_name: var("DYNAMODB_TABLE_NAME").unwrap(),
        }
    }
}

fn extract_string(
    item: &HashMap<String, AttributeValue>,
    key: &str,
) -> Result<String, DataStoreError> {
    match item.get(key) {
        Some(val) => match val.as_s() {
            Ok(val) => Ok(val.clone()),
            Err(err) => {
                println!("Couldn't parse {key} as string: {:?}", err);
                Err(DataTypeError)
            }
        },
        None => {
            println!("Missing field {key}");
            Err(DataTypeError)
        }
    }
}

fn load_url_record(result: GetItemOutput) -> Result<UrlRecord, DataStoreError> {
    match result.item {
        Some(item) => {
            let short_code = extract_string(&item, "short_code")?;
            let url = extract_string(&item, "url")?;
            Ok(UrlRecord { short_code, url })
        }
        None => Err(NotFound),
    }
}

#[async_trait]
impl DataStore for DynamoDbDataStore {
    async fn get(&self, short_code: &str) -> Result<UrlRecord, DataStoreError> {
        match self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("short_code", S(String::from(short_code)))
            .send()
            .await
        {
            Ok(result) => load_url_record(result),
            Err(err) => {
                println!(
                    "DynamoDB get {short_code} failed: {}",
                    DisplayErrorContext(err)
                );
                Err(ConnectionError)
            }
        }
    }

    async fn put(&self, short_code: &str, url: &str) -> Result<(), DataStoreError> {
        match self
            .client
            .put_item()
            .table_name(&self.table_name)
            .item("short_code", S(String::from(short_code)))
            .item("url", S(String::from(url)))
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                println!(
                    "DynamoDb put {short_code} => {url} failed: {}",
                    DisplayErrorContext(err)
                );
                Err(ConnectionError)
            }
        }
    }

    async fn exists(&self, short_code: &str) -> Result<bool, DataStoreError> {
        match self.get(short_code).await {
            Ok(_) => Ok(true),
            Err(NotFound) => Ok(false),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::types::AttributeValue::N;
    #[test]
    fn test_load_url_record() {
        let mut record = HashMap::new();
        record.insert(String::from("short_code"), S(String::from("asdf")));
        record.insert(String::from("url"), S(String::from("https://google.com")));

        let output = GetItemOutput::builder().set_item(Some(record)).build();

        let result = load_url_record(output).unwrap();
        assert_eq!(result.short_code, "asdf");
        assert_eq!(result.url, "https://google.com");
    }

    #[test]
    fn test_load_url_record_missing() {
        let output = GetItemOutput::builder().build();
        let error = load_url_record(output).err().unwrap();
        assert_eq!(error, NotFound);
    }

    #[test]
    fn test_extract_string_missing() {
        let map = HashMap::new();
        let error = extract_string(&map, "asdf").err().unwrap();
        assert_eq!(error, DataTypeError);
    }

    #[test]
    fn test_extract_string_wrong_type() {
        let mut map = HashMap::new();
        map.insert(String::from("asdf"), N(String::from("42")));

        let error = extract_string(&map, "asdf").err().unwrap();
        assert_eq!(error, DataTypeError);
    }
}
