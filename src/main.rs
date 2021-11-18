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

fn build_operators(opts: &options::Opts) -> Vec<Box<dyn queries::Operator>> {
    let mut operators: Vec<Box<dyn queries::Operator>> = Vec::new();
    if opts.start_time.is_some() || opts.end_time.is_some() {
        operators.push(Box::new(queries::TimespanFilter::new(
            opts.start_time,
            opts.end_time,
        )));
    }
    if opts.app.is_some() {
        operators.push(Box::new(queries::SimpleFieldFilter::new(
            "cloud_RoleName".to_owned(),
            opts.app.clone().unwrap(),
        )))
    }
    if opts.operation.is_some() {
        operators.push(Box::new(queries::SimpleFieldFilter::new(
            "operation_Name".to_owned(),
            opts.operation.clone().unwrap(),
        )))
    }
    operators
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = options::cli_opts(std::env::args())?;

    let operators = build_operators(&opts);

    let base_path = format!("{}/v1", ENDPOINT);
    let http_client = azure_core::new_http_client();
    let token_credential = Box::new(AzureCliCredential {});
    let config = azure_svc_applicationinsights::config(http_client, token_credential)
        .base_path(base_path)
        .token_credential_resource(ENDPOINT)
        .build();
    let querier = |mut operators: Vec<Box<dyn queries::Operator + 'static>>| async {
        let body = QueryBody {
            query: queries::tabular_expression("traces", &operators),
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
        if opts.follow {
            if last_message_ts.is_some() {
                operators[0]
                    .as_any()
                    .downcast_mut::<queries::TimespanFilter>()
                    .unwrap()
                    .advance_start(last_message_ts);
            }
            Ok(operators)
        } else {
            Err(anyhow!(AzTailError::Break))
        }
    };
    let err = util::repeater(Duration::from_secs(10), operators, querier).await;
    eprintln!("Failed {:?}", err);
    Ok(())
}
