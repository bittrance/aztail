use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use clap::Parser;
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[clap(version = "1.0")]
pub struct Opts {
    #[clap(long)]
    pub app_id: String,
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub start_time: Option<DateTime<FixedOffset>>,
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub end_time: Option<DateTime<FixedOffset>>,
    #[clap(short, long)]
    pub follow: bool,
}

fn parse_ts(input: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(input).map_err(|e| anyhow!(e))
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
    return Ok(opts);
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
