use metrics_exporter_prometheus::PrometheusBuilder;
use oxide_server::data_store::dynamodb::DynamoDbDataStore;
use oxide_server::{router, RandomShortCodeGenerator};
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::fmt::format::FmtSpan;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() {
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Couldn't install Prometheus recorder");

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .json()
        .init();

    dotenv::dotenv().ok();
    let bind_addr = "0.0.0.0:3000";

    let store = Arc::new(DynamoDbDataStore::new().await);

    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    let router = router(Arc::new(RandomShortCodeGenerator), store, prometheus_handle);

    tracing::info!("Listening on {bind_addr}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
