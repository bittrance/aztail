use crate::options::Opts;
use crate::source::{Level, LogEntry};
use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use colored::{Color, Colorize};
use serde_json::to_string_pretty;
use std::cell::RefCell;
use std::io::Write;

pub trait Presenter {
    fn present(&self, row: &LogEntry) -> Result<()>;
}

fn readable_timestamp(timestamp: DateTime<FixedOffset>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S%.3fZ").to_string()
}

fn message_color(log_entry: &LogEntry) -> Option<Color> {
    match log_entry.level() {
        Level::Verbose | Level::Info => None,
        Level::Warn => Some(Color::Yellow),
        Level::Error => Some(Color::BrightRed),
    }
}

pub struct PrettyJsonPresenter {}

impl Presenter for PrettyJsonPresenter {
    fn present(&self, log_entry: &LogEntry) -> Result<()> {
        println!("{}", to_string_pretty(&log_entry.raw())?);
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
            show_app: opts.function_app.len() != 1,
            show_operation: opts.function.len() != 1,
            output: Box::new(RefCell::new(output)),
        }
    }
}

impl Presenter for ColorTextPresenter {
    fn present(&self, log_entry: &LogEntry) -> Result<()> {
        let mut output = RefCell::borrow_mut(&self.output);
        let timestamp = readable_timestamp(log_entry.timestamp()).green();
        write!(output, "{}  ", timestamp)?;
        if self.show_app {
            let app = log_entry.group().magenta();
            write!(output, "{}  ", app)?;
        }
        if self.show_operation {
            let operation = log_entry.unit().cyan();
            write!(output, "{}  ", operation)?;
        }
        let message = match message_color(log_entry) {
            Some(color) => log_entry.message().color(color),
            None => log_entry.message().clear(),
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
    use crate::assembly::functions::traces_row_to_entry;
    use crate::examples::{traces_functions_row, T1};
    use crate::options::cli_opts;
    use crate::output::Presenter;
    use crate::testing::*;
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
        presenter.present(&log_entry(T1)).unwrap();
        let res = String::from_utf8(buf.take()).unwrap();
        assert_that(&res).contains("ze-app");
        assert_that(&res).contains("ze-operation");
        assert_that(&res).contains("ze-message");
    }

    #[test]
    fn naming_the_function_excludes_it_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--function", "ze-operation"])).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&log_entry(T1)).unwrap();
        assert_that(&String::from_utf8(buf.take()).unwrap()).does_not_contain("ze-operation");
    }

    #[test]
    fn naming_the_app_excludes_if_from_the_log() {
        let opts = cli_opts(base_args().chain(vec!["--function-app", "ze-app"])).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&log_entry(T1)).unwrap();
        assert_that(&String::from_utf8(buf.take()).unwrap()).does_not_contain("ze-app");
    }

    #[test]
    fn logs_have_color() {
        let mut row = traces_functions_row();
        row.insert("severityLevel".to_owned(), json!(2));
        let entry = traces_row_to_entry(row);
        let opts = cli_opts(base_args()).unwrap();
        let buf = Rc::new(RefCell::new(Vec::new()));
        let output = WriterWrapper { buf: buf.clone() };
        let presenter = ColorTextPresenter::new(output, &opts);
        presenter.present(&entry).unwrap();
        let res = String::from_utf8(buf.take()).unwrap();
        assert_that(&res).contains(&format!("{}", "ze-message".yellow()).as_ref());
    }
}
