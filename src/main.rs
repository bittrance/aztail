use crate::kusto::{Operator, Ordering, Query, SimpleFieldFilter, TimespanFilter};
use crate::output::{ColorTextPresenter, Presenter, PrettyJsonPresenter};
use crate::source::{appsinsight::AppInsights, LogSource};
use anyhow::Result;
use std::io::stdout;
use std::time::Duration;
use thiserror::Error;

mod kusto;
mod options;
mod output;
mod querier;
mod source;
#[cfg(test)]
mod testing;
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

fn build_operators(opts: &options::Opts) -> Vec<Box<dyn Operator>> {
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if opts.start_time.is_some() || opts.end_time.is_some() {
        operators.push(Box::new(TimespanFilter::new(
            opts.start_time,
            opts.end_time,
        )));
    }
    if opts.app.is_some() {
        operators.push(Box::new(SimpleFieldFilter::new(
            "cloud_RoleName".to_owned(),
            opts.app.clone().unwrap(),
        )));
    }
    if opts.operation.is_some() {
        operators.push(Box::new(SimpleFieldFilter::new(
            "operation_Name".to_owned(),
            opts.operation.clone().unwrap(),
        )));
    }
    operators.push(Box::new(Ordering {}));
    operators
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = options::cli_opts(std::env::args())?;
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let operators = build_operators(&opts);
    let query = Query::new("traces".to_owned(), operators);
    let log_source: Box<dyn LogSource> = Box::new(AppInsights::new(query, opts.clone()));
    let presenter = build_presenter(&opts);
    match util::repeater(
        Duration::from_secs(10),
        (vec![log_source], presenter, opts.follow),
        querier::querier,
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
    use crate::options::cli_opts;
    use crate::testing::base_args;
    use crate::Ordering;
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
