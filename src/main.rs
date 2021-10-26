use azure_identity::token_credentials::AzureCliCredential;
use azure_svc_applicationinsights::{models::QueryBody, operations::query};
use clap::Parser;
use serde_json::{map::Map, to_string_pretty, value::Value};
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const ENDPOINT: &str = "https://api.applicationinsights.io";

mod queries;

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(long)]
    app_id: String,
    #[clap(long)]
    query: String,
    timespan: Option<String>,
}

async fn repeater<T, F, E, Fut>(interval: Duration, initial: T, work: F) -> E
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut value = initial;
    loop {
        let next_repetition = Instant::now() + interval;
        match work(value).await {
            Ok(v) => value = v,
            Err(err) => return err,
        }
        if Instant::now() < next_repetition {
            sleep(next_repetition - Instant::now()).await;
        }
    }
}

fn present_row(row: Map<String, Value>) {
    println!("{}", to_string_pretty(&row).unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let query = QueryBody {
        query: opts.query,
        timespan: opts.timespan,
        applications: None,
    };

    let base_path = format!("{}/v1", ENDPOINT);
    let http_client = azure_core::new_http_client();
    let token_credential = Box::new(AzureCliCredential {});
    let config = azure_svc_applicationinsights::config(http_client, token_credential)
        .base_path(base_path)
        .token_credential_resource(ENDPOINT)
        .build();
    let querier = |query| async {
        let response = query::execute(&config, &opts.app_id, &query).await?;
        let unnamed = "unnamed".to_string();
        for table in response.tables {
            for row in table.rows.as_array().unwrap().iter() {
                let fields = table
                    .columns
                    .iter()
                    .map(|c| c.name.as_ref().unwrap_or_else(|| &unnamed));
                let values = row.as_array().unwrap();
                let m: Map<String, Value> = fields.cloned().zip(values.iter().cloned()).collect();
                present_row(m);
            }
        }
        Ok::<_, query::execute::Error>(query)
    };
    let err = repeater(Duration::from_secs(5), query, querier).await;
    println!("{:?}", err);
    Ok(())
}

#[tokio::test]
async fn test_repeater() {
    let start = Instant::now();
    let res = repeater(Duration::from_millis(20), 0, |mut counter| async move {
        counter += 1;
        if counter == 5 {
            return Err(counter);
        }
        Ok(counter)
    })
    .await;
    assert_eq!(res, 5);
    assert!(Instant::now() - start > Duration::from_millis(80));
    assert!(Instant::now() - start < Duration::from_millis(150));
}

#[tokio::test]
async fn test_repeater_slow_worker() {
    let start = Instant::now();
    repeater(Duration::from_millis(5), 0, |mut counter| async move {
        sleep(Duration::from_millis(20)).await;
        counter += 1;
        if counter == 5 {
            return Err(counter);
        }
        Ok(counter)
    })
    .await;
    assert!(Instant::now() - start > Duration::from_millis(80));
}
