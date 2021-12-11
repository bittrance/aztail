use crate::queries::{Ordering, Query, TimespanFilter};
use crate::{appinsights::LogSource, output::Presenter};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{
    json,
    value::{Map, Value},
};
use std::sync::{Arc, Mutex};

pub fn row() -> Map<String, Value> {
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

pub struct TestSource {
    pub query: Query,
    pub results: Mutex<Vec<Map<String, Value>>>,
}

impl TestSource {
    pub fn with_example_data() -> Self {
        Self {
            query: some_query(),
            results: Mutex::new(vec![row()]),
        }
    }
}

#[async_trait]
impl LogSource for TestSource {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = Map<String, Value>>>> {
        let res = self.results.lock().unwrap().clone();
        Ok(Box::new(res.into_iter()))
    }
    fn get_query_mut(&mut self) -> &mut Query {
        &mut self.query
    }
}

pub struct TestPresenter {
    pub rows: Arc<Mutex<Vec<Map<String, Value>>>>,
}

impl TestPresenter {
    pub fn new() -> Self {
        Self {
            rows: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Presenter for TestPresenter {
    fn present(&self, row: &Map<String, Value>) -> Result<()> {
        self.rows.lock().unwrap().push(row.clone());
        Ok(())
    }
}

pub fn some_query() -> Query {
    Query::new(
        "ZeTable".to_owned(),
        vec![
            Box::new(TimespanFilter::new(
                "2021-11-20T05:18:30+00:00".parse().ok(),
                None,
            )),
            Box::new(Ordering),
        ],
    )
}
