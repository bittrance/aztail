use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, Local};
use chrono_english::{parse_date_string, Dialect};
use clap::Parser;
#[cfg(test)]
use spectral::prelude::*;
use std::ffi::OsString;

/// Query the "traces" table in a App Insights or Log Analytics workspace
/// and presents the log entries.
#[derive(Debug, Parser)]
#[clap(version = "1.0")]
pub struct Opts {
    /// The UUID of the App Insight or Log Analytics workspace where logs reside
    #[clap(long)]
    pub app_id: String,
    /// Retrieve logs newer than this. Can be RFC3339 or informal such as "yesterday"
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// Retrieve logs older than this. Can be RFC3339 or informal such as "30min ago"
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// Show only logs for a specific app
    #[clap(short, long)]
    pub app: Option<String>,
    /// Show only logs for a specific function
    #[clap(short, long)]
    pub operation: Option<String>,
    /// Tail a log query. Incompatible with --end-time
    #[clap(short, long)]
    pub follow: bool,
}

fn parse_ts(input: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(input)
        .or_else(|_| parse_date_string(input, Local::now().into(), Dialect::Us))
        .map_err(|e| anyhow!(e))
}

pub fn cli_opts<I, T>(args: I) -> Result<Opts>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let opts = Opts::try_parse_from(args)?;
    if opts.end_time.is_some() && opts.follow {
        return Err(anyhow!("Please use --end-time or --follow, but not both"));
    }
    Ok(opts)
}

#[cfg(test)]
fn base_args() -> impl Iterator<Item = &'static str> {
    vec![
        "aztail",
        "--app-id",
        "ze-app",
        "-s",
        "2021-10-31T23:50:00+00:00",
    ]
    .into_iter()
}

#[test]
fn cli_options_minimum_working() {
    let res = cli_opts(base_args()).expect("parsing failed");
    assert_eq!(
        res.start_time,
        Some(DateTime::parse_from_rfc3339("2021-10-31T23:50:00+00:00").unwrap())
    );
    assert_eq!(res.app_id, "ze-app");
}

#[test]
fn cli_options_end_time_and_follow_incompatible() {
    let args = base_args().chain(vec!["-e", "2021-10-31T23:55:00+00:00", "-f"]);
    let res = cli_opts(args);
    assert!(format!("{:?}", res.unwrap_err()).contains("--end-time or --follow"));
}

#[test]
fn colloquial_end_time() {
    let args = base_args().chain(vec!["--end-time=-20m"]);
    let res = cli_opts(args);
    assert_that(&res).is_ok().map(|o| &o.end_time).is_some();
}
