use crate::options::Opts;
use anyhow::Result;
use chrono::DateTime;
use colored::{Color, Colorize};
use serde_json::{map::Map, to_string_pretty, value::Value};
use std::io::Write;

fn unwrap_as_str(value: Option<&Value>) -> &str {
    value.unwrap().as_str().unwrap()
}

fn readable_timestamp(value: Option<&Value>) -> String {
    let timestamp = DateTime::parse_from_rfc3339(value.and_then(|v| v.as_str()).unwrap()).unwrap();
    timestamp.format("%Y-%m-%d %H:%M:%S%.3fZ").to_string()
}

fn message_color(row: &Map<String, Value>) -> Option<Color> {
    let severity = row.get("severityLevel").unwrap().as_i64();
    if severity == Some(1) {
        None
    } else if severity == Some(2) {
        Some(Color::Yellow)
    } else if severity >= Some(3) {
        Some(Color::BrightRed)
    } else {
        Some(Color::Magenta)
    }
}

pub fn render_pretty_json(row: &Map<String, Value>) -> Result<()> {
    println!("{}", to_string_pretty(row)?);
    Ok(())
}

pub fn render_text_line<T>(row: &Map<String, Value>, output: &mut T, opts: &Opts) -> Result<()>
where
    T: Write,
{
    write!(
        output,
        "{}  ",
        readable_timestamp(row.get("timestamp")).green()
    )?;
    if opts.app.is_none() {
        write!(
            output,
            "{}  ",
            unwrap_as_str(row.get("cloud_RoleName")).magenta()
        )?;
    }
    if opts.operation.is_none() {
        write!(
            output,
            "{}  ",
            unwrap_as_str(row.get("operation_Name")).cyan()
        )?;
    }
    let message = match message_color(row) {
        Some(color) => unwrap_as_str(row.get("message")).color(color),
        None => unwrap_as_str(row.get("message")).clear(),
    };
    writeln!(output, "{}", message)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::options::{base_args, cli_opts};
    use colored::Colorize;
    use serde_json::json;
    use serde_json::{map::Map, value::Value};
    use speculoos::prelude::*;

    fn row() -> Map<String, Value> {
        json!({
            "timestamp": "2021-11-20T06:18:30+00:00",
            "cloud_RoleName": "ze-app",
            "operation_Name": "ze-operation",
            "message": "ze-message",
            "severityLevel": 1,
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

    #[test]
    fn logs_have_color() {
        let mut entry = row();
        entry.insert("severityLevel".to_owned(), json!(2));
        let opts = cli_opts(base_args()).unwrap();
        let mut output: Vec<u8> = Vec::new();
        super::render_text_line(&entry, &mut output, &opts).unwrap();
        let res = String::from_utf8(output).unwrap();
        assert_that(&res).contains(&format!("{}", "ze-message".yellow()).as_ref());
    }
}
