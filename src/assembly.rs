use crate::options::Opts;
use crate::source::LogSource;
use chrono::DateTime;
use chrono::FixedOffset;
use serde_json::Value;
use std::iter::empty;

pub mod apim;
pub mod container_apps;
pub mod functions;

pub fn unwrap_as_rfc3339(value: Option<&Value>) -> DateTime<FixedOffset> {
    value
        .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
        .unwrap()
}

pub fn unwrap_as_str(value: Option<&Value>) -> &str {
    value.unwrap().as_str().unwrap()
}

pub fn build_sources(opts: &Opts) -> Vec<Box<dyn LogSource>> {
    empty()
        .chain(apim::opsinsights(opts))
        .chain(apim::appinsights(opts))
        .chain(functions::opsinsights(opts))
        .chain(functions::appinsights(opts))
        .chain(container_apps::opsinsights(opts))
        .collect()
}
