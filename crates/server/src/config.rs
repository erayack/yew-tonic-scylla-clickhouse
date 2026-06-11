use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub scylla_uri: String,
    pub clickhouse_url: String,
    pub clickhouse_database: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr = env::var("SERVER_BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
            .parse()?;

        Ok(Self {
            bind_addr,
            scylla_uri: env::var("SCYLLA_URI").unwrap_or_else(|_| "scylla:9042".to_string()),
            clickhouse_url: env::var("CLICKHOUSE_URL")
                .unwrap_or_else(|_| "http://clickhouse:8123".to_string()),
            clickhouse_database: env::var("CLICKHOUSE_DATABASE")
                .unwrap_or_else(|_| "default".to_string()),
            clickhouse_user: env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string()),
            clickhouse_password: env::var("CLICKHOUSE_PASSWORD").unwrap_or_default(),
        })
    }
}
