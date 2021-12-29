use crate::assembly::{unwrap_as_rfc3339, unwrap_as_str};
use crate::kusto::{EndsWith, Filter, Operator, Or, Ordering, Query, StartsWith, Timespan};
use crate::options::{Opts, Service};
use crate::source::{appsinsight::AppInsights, opsinsight::OpsLogs, Level, LogEntry, LogSource};
use serde_json::{json, Map, Value};

pub fn appinsights_api_management(opts: &Opts) -> impl IntoIterator<Item = Box<dyn LogSource>> {
    if opts.app_id.is_none() || !opts.requested_services().contains(&Service::APIManagement) {
        return None;
    }
    let timespan = Timespan::new("timestamp".to_owned(), opts.start_time, opts.end_time);
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if !opts.api_name.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.api_name
                .iter()
                .cloned()
                .map(|n| StartsWith::boxed("cloud_RoleName".to_owned(), format!("{}.", n)))
                .collect(),
        )));
    }
    if !opts.api_operation.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.api_name
                .iter()
                .cloned()
                .map(|n| EndsWith::boxed("operation_Name".to_owned(), format!(" {}", n)))
                .collect(),
        )));
    }
    operators.push(Ordering::boxed("timestamp".to_owned()));
    let query = Query::new("requests".to_owned(), timespan, operators);
    Some(AppInsights::boxed(
        query,
        Box::new(appinsights_requests_row_to_entry),
        opts.clone(),
    ))
}

pub fn appinsights_requests_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = unwrap_as_rfc3339(row.get("timestamp"));
    let group = unwrap_as_str(row.get("cloud_RoleName"))
        .split('.')
        .next()
        .unwrap()
        .to_owned();
    let unit = unwrap_as_str(row.get("operation_Name"))
        .rsplit(' ')
        .next()
        .unwrap()
        .to_owned();
    let level = if row.get("success").unwrap() == &json!("True") {
        Level::Info
    } else {
        Level::Warn
    };
    let message = appsinsights_requests_message_line(&row);
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

fn appsinsights_requests_message_line(row: &Map<String, Value>) -> String {
    let url = unwrap_as_str(row.get("url"));
    let status_code = unwrap_as_str(row.get("resultCode"));
    let dimensions: Map<String, Value> =
        serde_json::from_str(unwrap_as_str(row.get("customDimensions"))).unwrap();
    if url.is_empty() {
        if row.get("Message").is_some() {
            unwrap_as_str(row.get("Message")).to_string()
        } else {
            let duration = unwrap_as_str(dimensions.get("FunctionExecutionTimeMs"));
            let reason = unwrap_as_str(dimensions.get("TriggerReason"));
            format!("\"{}\" result {} took {} ms", reason, status_code, duration)
        }
    } else {
        let client_ip = unwrap_as_str(row.get("client_IP"));
        let method = unwrap_as_str(dimensions.get("HTTP Method"));
        let measurements: Map<String, Value> =
            serde_json::from_str(unwrap_as_str(row.get("customMeasurements"))).unwrap();
        let response_size = measurements.get("Response Size").unwrap();
        let duration = row.get("duration").unwrap().as_f64().unwrap();
        format!(
            "{} {} \"{}\" {} {} {}",
            client_ip, method, url, status_code, response_size, duration
        )
    }
}

pub fn opsinsights_api_management(opts: &Opts) -> impl IntoIterator<Item = Box<dyn LogSource>> {
    if opts.workspace.is_none() || !opts.requested_services().contains(&Service::APIManagement) {
        return None;
    }
    let timespan = Timespan::new("TimeGenerated".to_owned(), opts.start_time, opts.end_time);
    let mut operators: Vec<Box<dyn Operator>> = Vec::new();
    if !opts.api_name.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.api_name
                .iter()
                .cloned()
                .map(|n| StartsWith::boxed("AppRoleName".to_owned(), format!("{}.", n)))
                .collect(),
        )));
    }
    if !opts.api_operation.is_empty() {
        operators.push(Filter::boxed(Or::new(
            opts.api_name
                .iter()
                .cloned()
                .map(|n| EndsWith::boxed("OperationName".to_owned(), format!(" {}", n)))
                .collect(),
        )));
    }
    operators.push(Ordering::boxed("TimeGenerated".to_owned()));
    let query = Query::new("AppRequests".to_owned(), timespan, operators);
    Some(OpsLogs::boxed(
        query,
        Box::new(opsinsights_requests_row_to_entry),
        opts.clone(),
    ))
}

pub fn opsinsights_requests_row_to_entry(row: Map<String, Value>) -> LogEntry {
    let timestamp = unwrap_as_rfc3339(row.get("TimeGenerated"));
    let group = unwrap_as_str(row.get("AppRoleName"))
        .split('.')
        .next()
        .unwrap()
        .to_owned();
    let unit = unwrap_as_str(row.get("OperationName"))
        .rsplit(' ')
        .next()
        .unwrap()
        .to_owned();
    let level = if row.get("Success").unwrap() == &json!("True") {
        Level::Info
    } else {
        Level::Warn
    };
    let message = opsinsights_requests_message_line(&row);
    LogEntry {
        timestamp,
        group,
        unit,
        level,
        message,
        raw: row,
    }
}

fn opsinsights_requests_message_line(row: &Map<String, Value>) -> String {
    let url = unwrap_as_str(row.get("Url"));
    let client_ip = unwrap_as_str(row.get("ClientIP"));
    let status_code = unwrap_as_str(row.get("ResultCode"));
    let dimensions: Map<String, Value> =
        serde_json::from_str(unwrap_as_str(row.get("Properties"))).unwrap();
    if url.is_empty() {
        if row.get("Message").is_some() {
            unwrap_as_str(row.get("Message")).to_string()
        } else {
            let duration = unwrap_as_str(dimensions.get("FunctionExecutionTimeMs"));
            let reason = unwrap_as_str(dimensions.get("TriggerReason"));
            format!("\"{}\" result {} took {} ms", reason, status_code, duration)
        }
    } else {
        let measurements: Map<String, Value> =
            serde_json::from_str(unwrap_as_str(row.get("Measurements"))).unwrap();
        let method = unwrap_as_str(dimensions.get("HTTP Method"));
        let response_size = measurements.get("Response Size").unwrap();
        let duration = row.get("DurationMs").unwrap().as_f64().unwrap();
        format!(
            "{} {} \"{}\" {} {} {}",
            client_ip, method, url, status_code, response_size, duration
        )
    }
}

#[cfg(test)]
mod test {
    use super::appinsights_requests_row_to_entry;
    use super::{appinsights_requests_row_to_entry, opsinsights_requests_row_to_entry};
    use crate::source::Level;
    use crate::{source::Level, testing::example_requests_row};
    use serde_json::Value;
    use speculoos::prelude::*;

    #[test]
    fn requests_row_to_entry_sets_log_level() {
        let mut row = example_requests_row();
        assert_that(&appinsights_requests_row_to_entry(row.clone()).level())
            .is_equal_to(Level::Info);
        row.insert("success".to_owned(), Value::String("False".to_owned()));
        assert_that(&appinsights_requests_row_to_entry(row).level()).is_equal_to(Level::Warn);
    }

    #[test]
    fn requests_row_to_entry_group_and_unit() {
        let res = appinsights_requests_row_to_entry(example_requests_row());
        assert_that(&res.group()).contains("aztail-apim");
        assert_that(&res.unit()).contains("get-ping");
    }

    #[test]
    fn requests_row_to_entry_contains_url() {
        let res = appinsights_requests_row_to_entry(example_requests_row());
        assert_that(&res.message()).contains("\"https://aztail-apim");
    }

    #[test]
    fn opsinsights_requests_message_line_function_style() {
        let row = apprequests_functions_row();
        let res = opsinsights_requests_row_to_entry(row);
        assert_that(&res.message()).does_not_contain("\"https://aztail-apim");
    }
}
