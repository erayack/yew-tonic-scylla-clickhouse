mod clickhouse;
mod config;
mod events;
mod grpc;
mod scylla;

use std::sync::Arc;

use clickhouse::ClickHouseStore;
use config::Config;
use events::{AnalyticsEventStore, EventRepository, OperationalEventStore};
use grpc::AppGrpcService;
use scylla::ScyllaStore;
use shared::app::app_service_server::AppServiceServer;
use tonic::transport::Server;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    info!(bind_addr = %config.bind_addr, "starting server");

    // Keep this starter wired directly: main -> gRPC -> events -> storage adapters.
    // Add broader app/domain/framework layers only after another slice makes them real.
    let scylla: Arc<dyn OperationalEventStore> =
        Arc::new(ScyllaStore::connect(&config.scylla_uri).await?);
    let clickhouse: Arc<dyn AnalyticsEventStore> =
        Arc::new(ClickHouseStore::connect(&config).await?);
    let events = Arc::new(EventRepository::new(scylla, clickhouse));
    let service = AppGrpcService::new(events);

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .add_service(tonic_web::enable(AppServiceServer::new(service)))
        .serve(config.bind_addr)
        .await?;

    Ok(())
}
