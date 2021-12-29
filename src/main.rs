use crate::assembly::build_sources;
use crate::output::{ColorTextPresenter, Presenter, PrettyJsonPresenter};
use anyhow::Result;
use std::io::stdout;
use std::time::Duration;
use thiserror::Error;

mod assembly;
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
    // Option parsing
    #[error("Use one of --app-id or --workspace")]
    AppInsightsOrLogAnalytics,
    #[error("Service exports to Log Analytics; please use --workspace")]
    LogAnalyticsService,
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

#[tokio::main]
async fn main() -> Result<()> {
    let opts = options::cli_opts(std::env::args())?;
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();
    let log_sources = build_sources(&opts);
    let presenter = build_presenter(&opts);
    match util::repeater(
        Duration::from_secs(10),
        (log_sources, presenter, opts.follow),
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
