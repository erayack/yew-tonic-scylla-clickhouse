use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Serialize;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    config::Config,
    events::{AnalyticsEvent, AnalyticsEventStore},
};

const INIT_SCHEMA: &str = include_str!("../../../migrations/clickhouse/001_init.sql");

pub struct ClickHouseStore {
    client: clickhouse::Client,
}

#[derive(Row, Serialize)]
struct EventAnalyticsRow {
    id: Uuid,
    name: String,
    created_at: i64,
}

impl ClickHouseStore {
    pub async fn connect(config: &Config) -> anyhow::Result<Self> {
        let mut client = clickhouse::Client::default()
            .with_url(&config.clickhouse_url)
            .with_database(&config.clickhouse_database)
            .with_user(&config.clickhouse_user);

        if !config.clickhouse_password.is_empty() {
            client = client.with_password(&config.clickhouse_password);
        }

        let store = Self { client };
        // Starter convenience: apply the checked-in local schema during connect.
        // Replace this adapter-local bootstrap with migration tooling before production use.
        store.bootstrap_schema().await?;
        Ok(store)
    }

    async fn bootstrap_schema(&self) -> anyhow::Result<()> {
        for statement in INIT_SCHEMA
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            self.client.query(statement).execute().await?;
        }
        Ok(())
    }

    pub async fn insert_event_analytics(
        &self,
        id: Uuid,
        name: &str,
        created_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let row = EventAnalyticsRow {
            id,
            name: name.to_string(),
            created_at: created_at.timestamp_millis(),
        };
        let mut insert = self.client.insert("events_analytics")?;
        insert.write(&row).await?;
        insert.end().await?;
        Ok(())
    }
}

#[async_trait]
impl AnalyticsEventStore for ClickHouseStore {
    async fn insert_analytics_event(&self, event: &AnalyticsEvent) -> anyhow::Result<()> {
        self.insert_event_analytics(event.id, event.name.as_str(), event.created_at)
            .await
    }
}
