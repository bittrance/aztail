use crate::assembly::functions::traces_row_to_entry;
use crate::kusto::{Ordering, Query, Timespan};
use crate::output::Presenter;
use crate::source::{LogEntry, LogSource};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

pub const T1: &str = "2021-11-20T06:18:30+00:00";
pub const T2: &str = "2021-11-20T06:18:31+00:00";
pub const T3: &str = "2021-11-20T06:18:32+00:00";
pub const T4: &str = "2021-11-20T06:18:33+00:00";

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

pub fn example_traces_row() -> Map<String, Value> {
    json!({
        "timestamp": T1,
        "cloud_RoleName": "ze-app",
        "operation_Name": "ze-operation",
        "message": "ze-message",
        "severityLevel": 1,
    })
    .as_object()
    .unwrap()
    .clone()
}

pub fn log_entry<'a>(timestamp: &'a str) -> LogEntry {
    let mut raw = example_traces_row();
    raw["timestamp"] = Value::String(timestamp.to_owned());
    traces_row_to_entry(raw)
}

pub fn example_requests_row() -> Map<String, Value> {
    json!({
        "client_Browser": "",
        "client_IP": "83.248.129.91",
        "client_Model": "",
        "client_OS": "",
        "client_Type": "PC",
        "cloud_RoleName": "aztail-apim.azure-api.net West Europe",
        "customDimensions": "{\"API Type\":\"http\",\"Subscription Name\":\"master\",\"Operation Name\":\"get-ping\",\"Region\":\"West Europe\",\"API Revision\":\"1\",\"Request Id\":\"e84f10ca-b9f8-40ab-8ed8-6e3588445262\",\"Service Name\":\"aztail-apim.azure-api.net\",\"Request-accept\":\"*/*\",\"Cache\":\"None\",\"Service Type\":\"API Management\",\"Response-content-length\":\"0\",\"API Name\":\"aztail-api\",\"HTTP Method\":\"GET\"}",
        "customMeasurements": "{\"Response Size\":93,\"Request Size\":0,\"Client Time (in ms)\":0}",
        "duration": 0.2486,
        "itemCount": 1,
        "name": "GET /example/",
        "operation_Name": "aztail-api;rev=1 - get-ping",
        "resultCode": "200",
        "session_Id": "",
        "source": "",
        "success": "True",
        "timestamp": "2021-12-22T22:56:48.164Z",
        "url": "https://aztail-apim.azure-api.net/example/?foo=bar",
        "user_AccountId": "",
        "user_AuthenticatedId": "",
        "user_Id": ""
    }).as_object().unwrap().clone()
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
