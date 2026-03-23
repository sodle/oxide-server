mod common;
use crate::common::{follow_short_code, shorten_url};
use axum_test::TestServer;
use metrics_exporter_prometheus::PrometheusBuilder;
use oxide_server::data_store::in_memory::InMemoryDataStore;
use oxide_server::{router, ShortCodeGenerator, ShortenUrlOutput};
use std::sync::{Arc, Mutex};

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
    let prometheus_handle = PrometheusBuilder::new().build_recorder().handle();

    TestServer::new(router(Arc::new(generator), store, prometheus_handle))
}

#[tokio::test]
async fn test_collision() {
    let mut codes = Vec::new();
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillconflict"));
    codes.push(String::from("thiswillnotconflict"));

    let server = test_scripted_server(codes).await;

    let first_short_code = shorten_url(&server, "https://google.com")
        .await
        .json::<ShortenUrlOutput>()
        .short_code;
    let second_short_code = shorten_url(&server, "https://scoott.blog")
        .await
        .json::<ShortenUrlOutput>()
        .short_code;

    assert_ne!(first_short_code, second_short_code);

    let visit_response = follow_short_code(&server, second_short_code.as_str()).await;
    assert_eq!(visit_response.status_code(), 302);
    assert_eq!(visit_response.header("Location"), "https://scoott.blog");
}
