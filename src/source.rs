use crate::kusto::Query;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use serde_json::{map::Map, value::Value};

pub mod appinsight;
pub mod opsinsight;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Verbose,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub group: String,
    pub unit: String,
    pub level: Level,
    pub message: String,
    pub raw: Map<String, Value>,
}

impl LogEntry {
    pub fn timestamp(&self) -> DateTime<FixedOffset> {
        self.timestamp
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn raw(&self) -> &Map<String, Value> {
        &self.raw
    }
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl PartialOrd for LogEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

#[async_trait]
#[allow(clippy::module_name_repetitions)]
pub trait LogSource {
    async fn stream(&self) -> Result<Box<dyn Iterator<Item = LogEntry>>>;
    fn get_query_mut(&mut self) -> &mut Query;
}

pub type Adapter = Box<dyn Fn(Map<String, Value>) -> LogEntry + Sync + Send>;
