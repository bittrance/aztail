use crate::options::Opts;
use crate::queries::Query;
use anyhow::Result;
use async_trait::async_trait;
use azure_identity::token_credentials::AzureCliCredential;
use azure_svc_applicationinsights::{
    config, models::QueryBody, operations::query, OperationConfig,
};
use chrono::{DateTime, FixedOffset};
use serde_json::{map::Map, value::Value};

const ENDPOINT: &str = "https://api.applicationinsights.io";

#[derive(Clone, Copy, Debug)]
pub enum Level {
    Verbose,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    group: String,
    unit: String,
    level: Level,
    message: String,
    raw: Map<String, Value>,
}

impl LogEntry {
    pub fn timestamp(&self) -> DateTime<FixedOffset> {
        self.timestamp
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn raw(&self) -> &Map<String, Value> {
        &self.raw
    }
}

fn unwrap_as_str(value: Option<&Value>) -> &str {
    value.unwrap().as_str().unwrap()
}

pub fn appinsights_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = row
        .get("timestamp")
        .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
        .unwrap();
    let group = unwrap_as_str(row.get("cloud_RoleName")).to_owned();
    let unit = unwrap_as_str(row.get("operation_Name")).to_owned();
    let level = match row.get("severityLevel").unwrap().as_i64() {
        Some(3) => Level::Error,
        Some(2) => Level::Warn,
        Some(1) | None => Level::Info,
        Some(_) => Level::Verbose,
    };
    let message = unwrap_as_str(row.get("message")).to_owned();
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl PartialOrd for LogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait LogSource {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>>;
    fn get_query_mut(&mut self) -> &mut Query;
}

pub struct AppInsights {
    config: OperationConfig,
    query: Query,
    opts: Opts,
}

impl AppInsights {
    pub fn new(query: Query, opts: Opts) -> Self {
        let base_path = format!("{}/v1", ENDPOINT);
        let http_client = azure_core::new_http_client();
        let token_credential = Box::new(AzureCliCredential {});
        let config = config(http_client, token_credential)
            .base_path(base_path)
            .token_credential_resource(ENDPOINT)
            .build();
        AppInsights {
            config,
            query,
            opts,
        }
    }
}

#[async_trait]
impl LogSource for AppInsights {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>> {
        let body = QueryBody {
            query: self.query.tabular_expression(),
            timespan: None,
            applications: None,
        };
        let response = query::execute(&self.config, &self.opts.app_id, &body).await?;
        let log_entries = response
            .tables
            .into_iter()
            .flat_map(|table| {
                let fields: Vec<String> = table
                    .columns
                    .into_iter()
                    .map(|c| c.name.unwrap_or_else(|| "unnamed".to_string()))
                    .collect();
                table
                    .rows
                    .as_array()
                    .cloned()
                    .unwrap()
                    .into_iter()
                    .map(move |row| {
                        fields
                            .clone()
                            .into_iter()
                            .zip(row.as_array().cloned().unwrap())
                            .collect::<Map<String, Value>>()
                    })
            })
            .map(appinsights_row_to_entry);
        Ok(Box::new(log_entries))
    }

    fn get_query_mut(&mut self) -> &mut Query {
        &mut self.query
    }
}
