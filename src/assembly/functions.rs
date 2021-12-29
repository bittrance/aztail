use serde_json::{Map, Value};

use crate::{
    kusto::{Eq, Filter, Operator, Or, Ordering, Query, Timespan},
    options::{Opts, Service},
    source::{appsinsight::AppInsights, opsinsight::OpsLogs, Level, LogEntry, LogSource},
};

use super::{unwrap_as_rfc3339, unwrap_as_str};

pub fn appinsights_functions(opts: &Opts) -> impl IntoIterator<Item = Box<dyn LogSource>> {
    if opts.app_id.is_none() || !opts.requested_services().contains(&Service::Functions) {
        return None;
    }
    let timespan = Timespan::new("timestamp".to_owned(), opts.start_time, opts.end_time);
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if !opts.api_name.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.function_app
                .iter()
                .cloned()
                .map(|n| Eq::boxed("cloud_RoleName".to_owned(), n))
                .collect(),
        )));
    }
    if !opts.api_operation.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.function
                .iter()
                .cloned()
                .map(|n| Eq::boxed("operation_Name".to_owned(), n))
                .collect(),
        )));
    }
    operators.push(Ordering::boxed("timestamp".to_owned()));
    let query = Query::new("traces".to_owned(), timespan, operators);
    Some(AppInsights::boxed(
        query,
        Box::new(traces_row_to_entry),
        opts.clone(),
    ))
}

pub fn traces_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = unwrap_as_rfc3339(row.get("timestamp"));
    let group = unwrap_as_str(row.get("cloud_RoleName")).to_owned();
    let unit = unwrap_as_str(row.get("operation_Name")).to_owned();
    let level = match row.get("severityLevel").unwrap().as_i64() {
        Some(3) => Level::Error,
        Some(2) => Level::Warn,
        Some(1) | None => Level::Info,
        Some(_) => Level::Verbose,
    };
    let message = unwrap_as_str(row.get("message")).to_owned();
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

pub fn opsinsights_functions(opts: &Opts) -> impl IntoIterator<Item = Box<dyn LogSource>> {
    if opts.workspace.is_none() || !opts.requested_services().contains(&Service::Functions) {
        return None;
    }
    let timespan = Timespan::new("TimeGenerated".to_owned(), opts.start_time, opts.end_time);
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if !opts.api_name.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.function_app
                .iter()
                .cloned()
                .map(|n| Eq::boxed("AppRoleName".to_owned(), n))
                .collect(),
        )));
    }
    if !opts.api_operation.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.function
                .iter()
                .cloned()
                .map(|n| Eq::boxed("OperationName".to_owned(), n))
                .collect(),
        )));
    }
    operators.push(Ordering::boxed("TimeGenerated".to_owned()));
    let query = Query::new("AppTraces".to_owned(), timespan, operators);
    Some(OpsLogs::boxed(
        query,
        Box::new(apptraces_row_to_entry),
        opts.clone(),
    ))
}

fn apptraces_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = unwrap_as_rfc3339(row.get("TimeGenerated"));
    let group = unwrap_as_str(row.get("AppRoleName")).to_owned();
    let unit = unwrap_as_str(row.get("OperationName")).to_owned();
    let level = match row.get("SeverityLevel").unwrap().as_i64() {
        Some(3) => Level::Error,
        Some(2) => Level::Warn,
        Some(1) | None => Level::Info,
        Some(_) => Level::Verbose,
    };
    let message = unwrap_as_str(row.get("Message")).to_owned();
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

#[cfg(test)]
mod test {
    use super::traces_row_to_entry;
    use crate::{examples::traces_functions_row, source::Level};
    use speculoos::prelude::*;

    #[test]
    fn apptraces_row_to_entry_sets_level() {
        let row = traces_functions_row();
        let res = traces_row_to_entry(row);
        assert_that(&res.level()).is_equal_to(Level::Info);
    }
}
