use crate::kusto::Query;
use crate::options::Opts;
use crate::source::{Adapter, LogEntry, LogSource};
use anyhow::Result;
use async_trait::async_trait;
use azure_identity::token_credentials::AzureCliCredential;
use azure_svc_operationalinsights::models::QueryBody;
use azure_svc_operationalinsights::{Client, ClientBuilder};
use serde_json::{map::Map, value::Value};
use std::sync::Arc;

const ENDPOINT: &str = "https://api.loganalytics.io";

pub struct OpsLogs {
    client: Client,
    query: Query,
    adapter: Arc<Adapter>,
    opts: Opts,
}

impl OpsLogs {
    pub fn new(query: Query, adapter: Adapter, opts: Opts) -> Self {
        let base_path = format!("{}/v1", ENDPOINT);
        let token_credential = Arc::new(AzureCliCredential {});
        let client = ClientBuilder::new(token_credential)
            .endpoint(base_path)
            .scopes(&[ENDPOINT])
            .build();
        OpsLogs {
            client,
            query,
            adapter: Arc::new(adapter),
            opts,
        }
    }

    pub fn boxed(query: Query, adapter: Adapter, opts: Opts) -> Box<Self> {
        Box::new(OpsLogs::new(query, adapter, opts))
    }
}

#[async_trait]
impl LogSource for OpsLogs {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>> {
        let body = QueryBody {
            query: format!("{}", self.query),
            timespan: None,
            workspaces: None,
        };
        let response = self
            .client
            .query()
            .execute(&self.opts.workspace.clone().unwrap(), body)
            .into_future()
            .await?;
        let adapter = self.adapter.clone();
        let log_entries = response
            .tables
            .into_iter()
            .flat_map(|table| {
                let fields: Vec<String> = table
                    .columns
                    .into_iter()
                    .map(|c| c.name.unwrap_or_else(|| "unnamed".to_string()))
                    .collect();
                table.rows.into_iter().map(move |row| {
                    fields
                        .clone()
                        .into_iter()
                        .zip(row.into_iter().map(Value::String))
                        .collect::<Map<String, Value>>()
                })
            })
            .map(move |row| adapter(row));
        Ok(Box::new(log_entries))
    }

    fn get_query_mut(&mut self) -> &mut Query {
        &mut self.query
    }
}
