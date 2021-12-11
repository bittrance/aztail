use crate::options::Opts;
use anyhow::Result;
use chrono::DateTime;
use colored::{Color, Colorize};
use serde_json::{map::Map, to_string_pretty, value::Value};
use std::cell::RefCell;
use std::io::Write;

pub trait Presenter {
    fn present(&self, row: &Map<String, Value>) -> Result<()>;
}

fn unwrap_as_str(value: Option<&Value>) -> &str {
    value.unwrap().as_str().unwrap()
}

fn readable_timestamp(value: Option<&Value>) -> String {
    let timestamp = DateTime::parse_from_rfc3339(value.and_then(Value::as_str).unwrap()).unwrap();
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

pub struct PrettyJsonPresenter {}

impl Presenter for PrettyJsonPresenter {
    fn present(&self, row: &Map<String, Value>) -> Result<()> {
        println!("{}", to_string_pretty(row)?);
        Ok(())
    }
}

pub struct ColorTextPresenter {
    show_app: bool,
    show_operation: bool,
    output: Box<RefCell<dyn Write>>,
}

impl<'a> ColorTextPresenter {
    pub fn new<W: 'static>(output: W, opts: &'a Opts) -> Self
    where
        W: Write + 'static,
    {
        Self {
            show_app: opts.app.is_none(),
            show_operation: opts.operation.is_none(),
            output: Box::new(RefCell::new(output)),
        }
    }
}

impl Presenter for ColorTextPresenter {
    fn present(&self, row: &Map<String, Value>) -> Result<()> {
        let mut output = RefCell::borrow_mut(&self.output);
        let timestamp = readable_timestamp(row.get("timestamp")).green();
        write!(output, "{}  ", timestamp)?;
        if self.show_app {
            let app = unwrap_as_str(row.get("cloud_RoleName")).magenta();
            write!(output, "{}  ", app)?;
        }
        if self.show_operation {
            let operation = unwrap_as_str(row.get("operation_Name")).cyan();
            write!(output, "{}  ", operation)?;
        }
        let message = match message_color(row) {
            Some(color) => unwrap_as_str(row.get("message")).color(color),
            None => unwrap_as_str(row.get("message")).clear(),
        };
        writeln!(output, "{}", message)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::Write;
    use std::rc::Rc;

    use super::ColorTextPresenter;
    use crate::options::{base_args, cli_opts};
    use crate::output::Presenter;
    use crate::testing::row;
    use colored::Colorize;
    use serde_json::json;
    use speculoos::prelude::*;

    struct WriterWrapper {
        buf: Rc<RefCell<Vec<u8>>>,
    }

    impl Write for WriterWrapper {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buf.borrow_mut().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.buf.borrow_mut().flush()
        }
    }

    #[test]
    fn default_textline() {
        let opts = cli_opts(base_args()).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&row()).unwrap();
        let res = String::from_utf8(buf.take()).unwrap();
        assert_that(&res).contains("ze-app");
        assert_that(&res).contains("ze-operation");
        assert_that(&res).contains("ze-message");
    }

    #[test]
    fn naming_the_function_excludes_it_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--operation", "ze-operation"])).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&row()).unwrap();
        assert_that(&String::from_utf8(buf.take()).unwrap()).does_not_contain("ze-operation");
    }

    #[test]
    fn naming_the_app_excludes_if_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--app", "ze-app"])).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&row()).unwrap();
        assert_that(&String::from_utf8(buf.take()).unwrap()).does_not_contain("ze-app");
    }

    #[test]
    fn logs_have_color() {
        let mut entry = row();
        entry.insert("severityLevel".to_owned(), json!(2));
        let opts = cli_opts(base_args()).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&entry).unwrap();
        let res = String::from_utf8(buf.take()).unwrap();
        assert_that(&res).contains(&format!("{}", "ze-message".yellow()).as_ref());
    }
}
