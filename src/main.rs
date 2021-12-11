use crate::output::{ColorTextPresenter, Presenter, PrettyJsonPresenter};
use anyhow::{anyhow, Result};
use appinsights::LogSource;
use chrono::{DateTime, FixedOffset};
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

fn build_presenter(opts: &options::Opts) -> Box<dyn Presenter> {
    match opts.format {
        options::OutputFormat::Json => Box::new(PrettyJsonPresenter {}) as Box<dyn Presenter>,
        options::OutputFormat::Text => {
            Box::new(ColorTextPresenter::new(stdout(), opts)) as Box<dyn Presenter>
        }
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
        )));
    }
    if opts.operation.is_some() {
        operators.push(Box::new(queries::SimpleFieldFilter::new(
            "operation_Name".to_owned(),
            opts.operation.clone().unwrap(),
        )));
    }
    operators.push(Box::new(queries::Ordering {}));
    operators
}

type QuerierArgs = (Box<dyn LogSource>, Box<dyn output::Presenter>, bool);

async fn querier((mut source, presenter, follow): QuerierArgs) -> Result<QuerierArgs> {
    let mut last_message_ts = None::<DateTime<FixedOffset>>;
    let log_entries = source.stream().await?;
    for row in log_entries {
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
        presenter.present(&row)?;
    }
    if follow {
        if last_message_ts.is_some() {
            source.get_query_mut().advance_start(last_message_ts);
        }
        Ok((source, presenter, follow))
    } else {
        Err(anyhow!(AzTailError::Break))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = options::cli_opts(std::env::args())?;
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let operators = build_operators(&opts);
    let query = queries::Query::new("traces".to_owned(), operators);
    let log_source: Box<dyn LogSource> =
        Box::new(appinsights::AppInsights::new(query, opts.clone()));
    let presenter = build_presenter(&opts);
    match util::repeater(
        Duration::from_secs(10),
        (log_source, presenter, opts.follow),
        querier,
    )
    .await
    {
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
