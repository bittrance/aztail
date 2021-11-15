use anyhow::{anyhow, Result};
use azure_identity::token_credentials::AzureCliCredential;
use azure_svc_applicationinsights::{models::QueryBody, operations::query};
use chrono::{DateTime, FixedOffset};
use serde_json::{map::Map, to_string_pretty, value::Value};
use std::time::Duration;
use thiserror::Error;

const ENDPOINT: &str = "https://api.applicationinsights.io";

mod options;
mod queries;
mod util;

#[derive(Error, Debug)]
pub enum AzTailError {
    #[error("No more entries")]
    Break,
}

fn present_row(row: &Map<String, Value>) {
    println!("{}", to_string_pretty(row).unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = options::cli_opts(std::env::args())?;

    let query = queries::QueryParams {
        item_type: "traces".to_owned(),
        start_time: opts.start_time,
        end_time: opts.end_time,
    };

    let base_path = format!("{}/v1", ENDPOINT);
    let http_client = azure_core::new_http_client();
    let token_credential = Box::new(AzureCliCredential {});
    let config = azure_svc_applicationinsights::config(http_client, token_credential)
        .base_path(base_path)
        .token_credential_resource(ENDPOINT)
        .build();
    let querier = |mut query| async {
        let body = QueryBody {
            query: queries::build_query(&query),
            timespan: None,
            applications: None,
        };
        let response = query::execute(&config, &opts.app_id, &body).await?;
        let unnamed = "unnamed".to_string();
        let mut last_message_ts = None::<DateTime<FixedOffset>>;
        for table in response.tables {
            for row in table.rows.as_array().unwrap().iter() {
                let fields = table
                    .columns
                    .iter()
                    .map(|c| c.name.as_ref().unwrap_or_else(|| &unnamed));
                let values = row.as_array().unwrap();
                let m: Map<String, Value> = fields.cloned().zip(values.iter().cloned()).collect();
                present_row(&m);
                if let Some(ts) = m
                    .get("timestamp")
                    .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
                {
                    last_message_ts = match last_message_ts {
                        None => Some(ts),
                        Some(prev_ts) if ts > prev_ts => Some(ts),
                        Some(prev_ts) => Some(prev_ts),
                    }
                }
            }
        }
        if last_message_ts.is_none() {
            // TODO: We should not follow beyond query.end_time
            if opts.follow {
                return Ok(query);
            }
            Err(anyhow!(AzTailError::Break))
        } else {
            query.start_time = last_message_ts;
            Ok(query)
        }
    };
    let err = util::repeater(Duration::from_secs(10), query, querier).await;
    eprintln!("Failed {:?}", err);
    Ok(())
}
