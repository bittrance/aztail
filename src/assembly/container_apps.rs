use serde_json::{Map, Value};

use crate::options::{Opts, Service};
use crate::{
    kusto::{Eq, Filter, Operator, Or, Ordering, Query, Timespan},
    source::{opsinsight::OpsLogs, Level, LogEntry, LogSource},
};

use super::{unwrap_as_rfc3339, unwrap_as_str};

fn opsinsights_container_apps_query(opts: &Opts) -> Query {
    let timespan = Timespan::new("TimeGenerated".to_owned(), opts.start_time, opts.end_time);
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if !opts.container_group.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.container_group
                .iter()
                .cloned()
                .map(|n| Eq::boxed("ContainerAppName_s".to_owned(), n))
                .collect(),
        )));
    }
    if !opts.container_name.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.container_name
                .iter()
                .cloned()
                .map(|n| Eq::boxed("ContainerName_s".to_owned(), n))
                .collect(),
        )));
    }
    operators.push(Ordering::boxed("TimeGenerated".to_owned()));
    Query::new("ContainerAppConsoleLogs_CL".to_owned(), timespan, operators)
}

fn container_apps_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = unwrap_as_rfc3339(row.get("TimeGenerated"));
    let group = unwrap_as_str(row.get("ContainerAppName_s")).to_owned();
    let unit = unwrap_as_str(row.get("ContainerName_s")).to_owned();
    let level = match row.get("Stream_s").unwrap().as_str() {
        Some(s) if s == "stderr" => Level::Error,
        Some(_) => Level::Info,
        None => Level::Info,
    };
    let message = unwrap_as_str(row.get("Log_s")).to_owned();
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

pub fn opsinsights(opts: &Opts) -> impl IntoIterator<Item = Box<dyn LogSource>> {
    if opts.workspace.is_none() || !opts.requested_services().contains(&Service::ContainerApps) {
        return None;
    }
    Some(OpsLogs::boxed(
        opsinsights_container_apps_query(opts),
        Box::new(container_apps_row_to_entry),
        opts.clone(),
    ))
}

#[cfg(test)]
mod test {
    use super::{container_apps_row_to_entry, opsinsights_container_apps_query};
    use crate::{
        examples::container_apps_row, options::cli_opts, source::Level,
        testing::opsinsights_base_args,
    };
    use speculoos::prelude::*;

    #[test]
    pub fn row_to_entry_sets_level() {
        let row = container_apps_row();
        let res = container_apps_row_to_entry(row);
        assert_that(&res.level()).is_equal_to(Level::Info);
    }

    #[test]
    pub fn generates_query() {
        let args = opsinsights_base_args().chain(vec!["--container-name", "ze-container"]);
        let opts = cli_opts(args).unwrap();
        let query = opsinsights_container_apps_query(&opts);
        assert_that!(query.to_string()).contains("ze-container");
    }
}
