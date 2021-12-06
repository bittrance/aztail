use anyhow::{anyhow, Result};
use appinsights::LogSource;
use chrono::{DateTime, FixedOffset};
use serde_json::{map::Map, value::Value};
use std::io::stdout;
use std::time::Duration;
use thiserror::Error;

mod appinsights;
mod options;
mod output;
mod queries;
mod util;

#[derive(Error, Debug, PartialEq)]
pub enum AzTailError {
    #[error("No more entries")]
    Break,
    #[error("Invalid output format: {0}")]
    InvalidOutputFormat(String),
}

fn present_row(row: &Map<String, Value>, opts: &options::Opts) -> Result<()> {
    match opts.format {
        options::OutputFormat::Json => output::render_pretty_json(row),
        options::OutputFormat::Text => output::render_text_line(row, &mut stdout(), opts),
    }
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
    operators.push(Box::new(queries::Ordering {}));
    operators
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = options::cli_opts(std::env::args())?;
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let operators = build_operators(&opts);
    let query = queries::Query::new("traces".to_owned(), operators);
    let log_source = appinsights::AppInsights::new(&opts);
    let querier = |mut query: queries::Query| async {
        let mut last_message_ts = None::<DateTime<FixedOffset>>;
        let log_entries = log_source.query(&query).await?;
        log_entries
            .inspect(|row| {
                if let Some(ts) = row
                    .get("timestamp")
                    .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
                {
                    last_message_ts = match last_message_ts {
                        None => Some(ts),
                        Some(prev_ts) if ts > prev_ts => Some(ts),
                        Some(prev_ts) => Some(prev_ts),
                    }
                }
            })
            .try_for_each(|row| present_row(&row, &opts))?;
        if opts.follow {
            if last_message_ts.is_some() {
                query.advance_start(last_message_ts);
            }
            Ok(query)
        } else {
            Err(anyhow!(AzTailError::Break))
        }
    };
    match util::repeater(Duration::from_secs(10), query, querier).await {
        ref err
            if err
                .downcast_ref::<AzTailError>()
                .filter(|e| e == &&AzTailError::Break)
                .is_some() =>
        {
            Ok(())
        }
        err => Err(err),
    }
}

#[cfg(test)]
mod test {
    use crate::options::{base_args, cli_opts};
    use crate::queries::Ordering;
    use speculoos::prelude::*;

    #[test]
    fn last_operator_is_ordering() {
        let opts = cli_opts(base_args()).unwrap();
        let res = super::build_operators(&opts);
        let ordering_pos = res.iter().position(|o| o.as_any().is::<Ordering>());
        assert_that(&ordering_pos)
            .is_some()
            .is_equal_to(res.len() - 1);
    }
}
