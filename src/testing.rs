use crate::assembly::functions::traces_row_to_entry;
use crate::examples::traces_functions_row;
use crate::kusto::{Ordering, Query, Timespan};
use crate::output::Presenter;
use crate::source::{LogEntry, LogSource};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, Mutex};

pub fn base_args() -> impl Iterator<Item = &'static str> {
    vec![
        "aztail",
        "--app-id",
        "ze-app",
        "-s",
        "2021-10-31T23:50:00+00:00",
    ]
    .into_iter()
}

pub fn opsinsights_base_args() -> impl Iterator<Item = &'static str> {
    vec![
        "aztail",
        "--workspace",
        "ze-app",
        "-s",
        "2021-10-31T23:50:00+00:00",
    ]
    .into_iter()
}

pub fn log_entry<'a>(timestamp: &'a str) -> LogEntry {
    let mut raw = traces_functions_row();
    raw["timestamp"] = Value::String(timestamp.to_owned());
    traces_row_to_entry(raw)
}

pub struct TestSource {
    pub query: Query,
    pub results: Mutex<Vec<LogEntry>>,
}

impl TestSource {
    pub fn with_example_data() -> Box<Self> {
        Box::new(Self {
            query: some_query(),
            results: Mutex::new(vec![log_entry("2021-11-20T06:18:30+00:00")]),
        })
    }

    pub fn with_rows(rows: Vec<LogEntry>) -> Box<Self> {
        Box::new(Self {
            query: some_query(),
            results: Mutex::new(rows),
        })
    }
}

#[async_trait]
impl LogSource for TestSource {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>> {
        let res = self.results.lock().unwrap().clone();
        Ok(Box::new(res.into_iter()))
    }

    fn get_query_mut(&mut self) -> &mut Query {
        &mut self.query
    }
}

pub struct TestPresenter {
    pub rows: Arc<Mutex<Vec<LogEntry>>>,
}

impl TestPresenter {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            rows: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn output_to<'a>(rows: &Arc<Mutex<Vec<LogEntry>>>) -> Box<Self> {
        Box::new(Self {
            rows: Arc::clone(rows),
        })
    }
}

impl Presenter for TestPresenter {
    fn present(&self, row: &LogEntry) -> Result<()> {
        self.rows.lock().unwrap().push(row.clone());
        Ok(())
    }
}

pub fn some_query() -> Query {
    Query::new(
        "ZeTable".to_owned(),
        Timespan::new(
            "timestamp".to_owned(),
            "2021-11-20T05:18:30+00:00".parse().ok(),
            None,
        ),
        vec![Box::new(Ordering::new("timestamp".to_owned()))],
    )
}
