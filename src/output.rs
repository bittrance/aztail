use crate::options::Opts;
use anyhow::Result;
use chrono::DateTime;
use serde_json::{map::Map, to_string_pretty, value::Value};
use std::io::Write;

fn value_as_bytes(value: Option<&Value>) -> &[u8] {
    value.unwrap().as_str().unwrap().as_bytes()
}

fn readable_timestamp(value: Option<&Value>) -> String {
    let timestamp = DateTime::parse_from_rfc3339(value.and_then(|v| v.as_str()).unwrap()).unwrap();
    timestamp.format("%Y-%m-%d %H:%M:%S%.3fZ").to_string()
}

pub fn render_pretty_json(row: &Map<String, Value>) -> Result<()> {
    println!("{}", to_string_pretty(row)?);
    Ok(())
}

pub fn render_text_line<T>(row: &Map<String, Value>, output: &mut T, opts: &Opts) -> Result<()>
where
    T: Write,
{
    output.write_all(readable_timestamp(row.get("timestamp")).as_bytes())?;
    output.write_all("  ".as_bytes())?;
    if opts.app.is_none() {
        output.write_all(value_as_bytes(row.get("cloud_RoleName")))?;
        output.write_all("  ".as_bytes())?;
    }
    if opts.operation.is_none() {
        output.write_all(value_as_bytes(row.get("operation_Name")))?;
        output.write_all("  ".as_bytes())?;
    }
    output.write_all(value_as_bytes(row.get("message")))?;
    output.write_all("\n".as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::options::{base_args, cli_opts};
    use serde_json::json;
    use serde_json::{map::Map, value::Value};
    use speculoos::prelude::*;

    fn row() -> Map<String, Value> {
        json!({
            "timestamp": "2021-11-20T06:18:30+00:00",
            "cloud_RoleName": "ze-app",
            "operation_Name": "ze-operation",
            "message": "ze-message",
        })
        .as_object()
        .unwrap()
        .clone()
    }

    #[test]
    fn default_textline() {
        let opts = cli_opts(base_args()).unwrap();
        let mut output: Vec<u8> = Vec::new();
        super::render_text_line(&row(), &mut output, &opts).unwrap();
        let res = String::from_utf8(output).unwrap();
        assert_that(&res).contains("ze-app");
        assert_that(&res).contains("ze-operation");
        assert_that(&res).contains("ze-message");
    }

    #[test]
    fn naming_the_function_excludes_it_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--operation", "ze-operation"])).unwrap();
        let mut output: Vec<u8> = Vec::new();
        super::render_text_line(&row(), &mut output, &opts).unwrap();
        assert_that(&String::from_utf8(output).unwrap()).does_not_contain("ze-operation");
    }

    #[test]
    fn naming_the_app_excludes_if_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--app", "ze-app"])).unwrap();
        let mut output: Vec<u8> = Vec::new();
        super::render_text_line(&row(), &mut output, &opts).unwrap();
        assert_that(&String::from_utf8(output).unwrap()).does_not_contain("ze-app");
    }
}
