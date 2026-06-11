use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use tonic::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventName(String);

impl EventName {
    pub fn parse(value: &str) -> anyhow::Result<Self> {
        let trimmed = value.trim();
        anyhow::ensure!(!trimmed.is_empty(), "event name must not be empty");
        Ok(Self(trimmed.to_owned()))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalEvent {
    pub id: Uuid,
    pub name: EventName,
    pub payload: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub name: EventName,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait OperationalEventStore: Send + Sync {
    async fn insert_operational_event(&self, event: &OperationalEvent) -> anyhow::Result<()>;
}

#[async_trait]
pub trait AnalyticsEventStore: Send + Sync {
    async fn insert_analytics_event(&self, event: &AnalyticsEvent) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct EventRepository {
    operational: Arc<dyn OperationalEventStore>,
    analytics: Arc<dyn AnalyticsEventStore>,
}

impl EventRepository {
    pub fn new(
        operational: Arc<dyn OperationalEventStore>,
        analytics: Arc<dyn AnalyticsEventStore>,
    ) -> Self {
        Self {
            operational,
            analytics,
        }
    }

    pub async fn create_event(&self, name: EventName, payload: &str) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let operational_event = OperationalEvent {
            id,
            name: name.clone(),
            payload: payload.to_string(),
            created_at,
        };
        let analytics_event = AnalyticsEvent {
            id,
            name,
            created_at,
        };

        self.operational
            .insert_operational_event(&operational_event)
            .await
            .context("failed to write event to operational store")?;

        self.analytics
            .insert_analytics_event(&analytics_event)
            .await
            .context("failed to write event to analytics store")?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RecordedCall {
        Operational(OperationalEvent),
        Analytics(AnalyticsEvent),
    }

    #[derive(Default)]
    struct MemoryOperationalStore {
        calls: Arc<Mutex<Vec<RecordedCall>>>,
        fail_with: Option<&'static str>,
    }

    impl MemoryOperationalStore {
        fn new(calls: Arc<Mutex<Vec<RecordedCall>>>) -> Self {
            Self {
                calls,
                fail_with: None,
            }
        }

        fn failing(calls: Arc<Mutex<Vec<RecordedCall>>>, message: &'static str) -> Self {
            Self {
                calls,
                fail_with: Some(message),
            }
        }
    }

    #[async_trait]
    impl OperationalEventStore for MemoryOperationalStore {
        async fn insert_operational_event(&self, event: &OperationalEvent) -> anyhow::Result<()> {
            if let Some(message) = self.fail_with {
                anyhow::bail!(message);
            }
            self.calls
                .lock()
                .expect("recorded call mutex poisoned")
                .push(RecordedCall::Operational(event.clone()));
            Ok(())
        }
    }

    #[derive(Default)]
    struct MemoryAnalyticsStore {
        calls: Arc<Mutex<Vec<RecordedCall>>>,
        fail_with: Option<&'static str>,
    }

    impl MemoryAnalyticsStore {
        fn new(calls: Arc<Mutex<Vec<RecordedCall>>>) -> Self {
            Self {
                calls,
                fail_with: None,
            }
        }

        fn failing(calls: Arc<Mutex<Vec<RecordedCall>>>, message: &'static str) -> Self {
            Self {
                calls,
                fail_with: Some(message),
            }
        }
    }

    #[async_trait]
    impl AnalyticsEventStore for MemoryAnalyticsStore {
        async fn insert_analytics_event(&self, event: &AnalyticsEvent) -> anyhow::Result<()> {
            if let Some(message) = self.fail_with {
                anyhow::bail!(message);
            }
            self.calls
                .lock()
                .expect("recorded call mutex poisoned")
                .push(RecordedCall::Analytics(event.clone()));
            Ok(())
        }
    }

    fn repository_with(
        operational: MemoryOperationalStore,
        analytics: MemoryAnalyticsStore,
    ) -> EventRepository {
        EventRepository::new(Arc::new(operational), Arc::new(analytics))
    }

    #[tokio::test]
    async fn create_event_writes_operational_then_analytics_with_consistent_fields() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let repository = repository_with(
            MemoryOperationalStore::new(Arc::clone(&calls)),
            MemoryAnalyticsStore::new(Arc::clone(&calls)),
        );

        let returned_id = repository
            .create_event(
                EventName::parse("signed_up").unwrap(),
                "{\"plan\":\"starter\"}",
            )
            .await
            .expect("event should be created");

        let calls = calls.lock().expect("recorded call mutex poisoned");
        assert_eq!(calls.len(), 2);
        let RecordedCall::Operational(operational) = &calls[0] else {
            panic!("first call should write operational storage");
        };
        let RecordedCall::Analytics(analytics) = &calls[1] else {
            panic!("second call should write analytics storage");
        };

        assert_eq!(returned_id, operational.id);
        assert_eq!(returned_id, analytics.id);
        assert_eq!(operational.name.as_str(), "signed_up");
        assert_eq!(analytics.name.as_str(), "signed_up");
        assert_eq!(operational.payload, "{\"plan\":\"starter\"}");
        assert_eq!(operational.created_at, analytics.created_at);
    }

    #[tokio::test]
    async fn create_event_does_not_write_analytics_when_operational_fails() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let repository = repository_with(
            MemoryOperationalStore::failing(Arc::clone(&calls), "scylla unavailable"),
            MemoryAnalyticsStore::new(Arc::clone(&calls)),
        );

        let error = repository
            .create_event(EventName::parse("signed_up").unwrap(), "{}")
            .await
            .expect_err("operational failure should be returned");

        assert!(
            format!("{error:#}").contains("failed to write event to operational store")
                && format!("{error:#}").contains("scylla unavailable"),
            "error should identify operational store failure: {error:#}"
        );
        assert!(
            calls
                .lock()
                .expect("recorded call mutex poisoned")
                .is_empty(),
            "analytics storage should not be called after operational failure"
        );
    }

    #[tokio::test]
    async fn create_event_returns_context_when_analytics_fails_after_operational_write() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let repository = repository_with(
            MemoryOperationalStore::new(Arc::clone(&calls)),
            MemoryAnalyticsStore::failing(Arc::clone(&calls), "clickhouse unavailable"),
        );

        let error = repository
            .create_event(EventName::parse("signed_up").unwrap(), "{}")
            .await
            .expect_err("analytics failure should be returned");

        assert!(
            format!("{error:#}").contains("failed to write event to analytics store")
                && format!("{error:#}").contains("clickhouse unavailable"),
            "error should identify analytics store failure: {error:#}"
        );
        let calls = calls.lock().expect("recorded call mutex poisoned");
        assert_eq!(calls.len(), 1);
        assert!(matches!(calls[0], RecordedCall::Operational(_)));
    }
}
