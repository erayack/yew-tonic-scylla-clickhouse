use chrono::{DateTime, Utc};
use scylla::{Session, SessionBuilder};
use tonic::async_trait;
use uuid::Uuid;

use crate::events::{OperationalEvent, OperationalEventStore};

const INIT_SCHEMA: &str = include_str!("../../../migrations/scylla/001_init.cql");

#[derive(Debug)]
pub struct ScyllaStore {
    session: Session,
}

impl ScyllaStore {
    pub async fn connect(uri: &str) -> anyhow::Result<Self> {
        let session = SessionBuilder::new().known_node(uri).build().await?;
        let store = Self { session };
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
            self.session.query_unpaged(statement, ()).await?;
        }
        Ok(())
    }

    pub async fn insert_event(
        &self,
        id: Uuid,
        name: &str,
        payload: &str,
        created_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.session
            .query_unpaged(
                "INSERT INTO app.events (id, name, payload, created_at) VALUES (?, ?, ?, ?)",
                (id, name, payload, created_at),
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl OperationalEventStore for ScyllaStore {
    async fn insert_operational_event(&self, event: &OperationalEvent) -> anyhow::Result<()> {
        self.insert_event(
            event.id,
            event.name.as_str(),
            &event.payload,
            event.created_at,
        )
        .await
    }
}
