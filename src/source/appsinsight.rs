use std::sync::Arc;

use crate::kusto::Query;
use crate::options::Opts;
use crate::source::{Adapter, LogEntry, LogSource};
use anyhow::Result;
use async_trait::async_trait;
use azure_identity::token_credentials::AzureCliCredential;
use azure_svc_applicationinsights::{models::QueryBody, Client, ClientBuilder};
use serde_json::value::{Map, Value};

const ENDPOINT: &str = "https://api.applicationinsights.io";

pub struct AppInsights {
    client: Client,
    query: Query,
    adapter: Arc<Adapter>,
    opts: Opts,
}

impl AppInsights {
    pub fn new(query: Query, adapter: Adapter, opts: Opts) -> Self {
        let base_path = format!("{}/v1", ENDPOINT);
        let token_credential = Arc::new(AzureCliCredential {});
        let client = ClientBuilder::new(token_credential)
            .endpoint(base_path)
            .scopes(&[ENDPOINT])
            .build();
        AppInsights {
            client,
            query,
            adapter: Arc::new(adapter),
            opts,
        }
    }

    pub fn boxed(query: Query, adapter: Adapter, opts: Opts) -> Box<dyn LogSource> {
        Box::new(AppInsights::new(query, adapter, opts))
    }
}

#[async_trait]
impl LogSource for AppInsights {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>> {
        let debug = self.opts.debug;
        let query = format!("{}", self.query);
        if debug {
            eprintln!("{}", query);
        }
        let body = QueryBody {
            query,
            timespan: None,
            applications: None,
        };
        let response = self
            .client
            .query()
            .execute(&self.opts.app_id.clone().unwrap(), body)
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
            .inspect(move |row| {
                if debug {
                    eprintln!("{:?}", row);
                }
            })
            .map(move |row| adapter(row));
        Ok(Box::new(log_entries))
    }

    fn get_query_mut(&mut self) -> &mut Query {
        &mut self.query
    }
}
