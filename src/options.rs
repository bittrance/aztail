use crate::AzTailError;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, Local};
use chrono_english::{parse_date_string, Dialect};
use clap::Parser;
use std::ffi::OsString;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Service {
    Functions,
    APIManagement,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
}

impl FromStr for OutputFormat {
    type Err = super::AzTailError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if lower == "text" {
            Ok(OutputFormat::Text)
        } else if lower == "json" {
            Ok(OutputFormat::Json)
        } else {
            Err(AzTailError::InvalidOutputFormat(lower))
        }
    }
}

/// Query tables in a App Insights or Log Analytics workspace and presents
/// the result as a human-readable log stream. When executed with only an
/// App ID or Workspace ID, aztail retrieves logs from all known services.
/// If one or more filter arguments are used, only logs matching those
/// filters will be retrieved. Multiple filters can be used and will
/// retrieve the union of matching data.

#[derive(Clone, Debug, Parser)]
#[clap(version = "0.8.0")]
pub struct Opts {
    /// The "Application ID" of the App Insight where logs reside
    #[clap(short, long)]
    pub app_id: Option<String>,
    /// The ID of the Log Analytics workspace where logs reside
    #[clap(short, long)]
    pub workspace: Option<String>,
    /// Retrieve logs newer than this. Can be RFC3339 or informal such as "yesterday"
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub start_time: Option<DateTime<FixedOffset>>,
    /// Retrieve logs older than this. Can be RFC3339 or informal such as "30min ago"
    #[clap(short, long, parse(try_from_str = parse_ts))]
    pub end_time: Option<DateTime<FixedOffset>>,
    /// Tail a log query. Incompatible with --end-time
    #[clap(short, long)]
    pub follow: bool,
    /// One of text, json
    #[clap(long, default_value = "text")]
    pub format: OutputFormat,
    /// Debug log all queries and all entries received
    #[clap(long)]
    pub debug: bool,

    // Azure Functions
    /// Show only logs for a specific app
    #[clap(long)]
    pub function_app: Vec<String>,
    /// Show only logs for a specific function
    #[clap(long)]
    pub function: Vec<String>,

    // Azure Container Instances
    /// Show only logs for a container group
    #[clap(long)]
    pub container_group: Vec<String>,
    /// Show only logs for a specific container
    #[clap(long)]
    pub container_name: Vec<String>,

    // Azure API management
    /// Show only logs for a particular API
    #[clap(long)]
    pub api_name: Vec<String>,
    /// Show only logs for a particular operation (regardless of owning API)
    #[clap(long)]
    pub api_operation: Vec<String>,
}

impl Opts {
    pub fn requested_services(&self) -> Vec<Service> {
        let mut requested_services = Vec::new();
        if !(self.api_name.is_empty() && self.api_operation.is_empty()) {
            requested_services.push(Service::APIManagement);
        }
        if !(self.function_app.is_empty() && self.function.is_empty()) {
            requested_services.push(Service::Functions);
        }
        if requested_services.is_empty() {
            requested_services.push(Service::APIManagement);
            requested_services.push(Service::Functions);
        }
        requested_services
    }
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
    if !(opts.app_id.is_some() ^ opts.workspace.is_some()) {
        return Err(anyhow!(AzTailError::AppInsightsOrLogAnalytics));
    }
    if opts.workspace.is_none()
        && (!opts.container_group.is_empty() || !opts.container_name.is_empty())
    {
        return Err(anyhow!(AzTailError::LogAnalyticsService));
    }
    Ok(opts)
}

#[cfg(test)]
mod test {
    use crate::{options::cli_opts, testing::base_args};
    use speculoos::prelude::*;

    #[test]
    fn cli_options_minimum_working() {
        let res = cli_opts(base_args()).expect("parsing failed");
        assert_that(&res.start_time).is_equal_to("2021-10-31T23:50:00+00:00".parse().ok());
        assert_that(&res.app_id).is_equal_to(&Some("ze-app".to_owned()));
    }

    #[test]
    fn appinsights_xor_workspace() {
        let res = cli_opts(vec!["aztail", "-s", "2021-10-31T23:50:00+00:00"].iter());
        assert_that(&format!("{:?}", res.unwrap_err())).contains("--app-id or --workspace");
        let res = cli_opts(base_args().chain(vec!["-w", "ze-workspace"]));
        assert_that(&format!("{:?}", res.unwrap_err())).contains("--app-id or --workspace");
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

    #[test]
    fn container_group_requires_workspace() {
        let res = cli_opts(base_args().chain(vec!["--container-group", "ze-group"]));
        assert_that(&format!("{:?}", res.unwrap_err())).contains("use --workspace");
    }
}
