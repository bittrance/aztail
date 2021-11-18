use chrono::prelude::*;
#[cfg(test)]
use spectral::prelude::*;
use std::any::Any;
use std::fmt::Debug;

pub trait Operator: Any + Debug {
    fn to_term(&self) -> String;
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct TimespanFilter {
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
}

impl TimespanFilter {
    pub fn new(
        start_time: Option<DateTime<FixedOffset>>,
        end_time: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self {
            start_time,
            end_time,
        }
    }

    pub fn advance_start(&mut self, start_time: Option<DateTime<FixedOffset>>) {
        self.start_time = start_time;
    }
}

impl Operator for TimespanFilter {
    fn to_term(&self) -> String {
        let mut op = String::new();
        if let Some(start_time) = self.start_time {
            op.push_str(&format!(" | where timestamp > datetime({:?})", start_time));
        }
        if let Some(end_time) = self.end_time {
            op.push_str(&format!(" | where timestamp < datetime({:?})", end_time));
        }
        op
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct SimpleFieldFilter {
    field: String,
    value: String,
}

impl SimpleFieldFilter {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }
}

impl Operator for SimpleFieldFilter {
    fn to_term(&self) -> String {
        format!(" | where {} == '{}'", self.field, self.value)
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Ordering;

impl Ordering {
    pub fn new() -> Self {
        Ordering
    }
}

impl Operator for Ordering {
    fn to_term(&self) -> String {
        " | sort by timestamp asc".to_string()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub fn tabular_expression<I>(source: &str, operators: I) -> String
where
    I: IntoIterator,
    I::Item: AsRef<dyn Operator>,
{
    let mut expr = source.to_owned();
    expr.push_str(
        &operators
            .into_iter()
            .map(|o| o.as_ref().to_term())
            .collect::<String>(),
    );
    expr
}

#[test]
fn timespan_starttime() {
    let timespan = TimespanFilter::new("2021-10-19T21:44:01.10Z".parse().ok(), None);
    assert_that(&timespan.to_term())
        .is_equal_to(" | where timestamp > datetime(2021-10-19T21:44:01.100+00:00)".to_owned());
}

#[test]
fn timespan_endtime() {
    let timespan = TimespanFilter::new(None, "2021-10-19T22:44:01.10Z".parse().ok());
    assert_that(&timespan.to_term())
        .is_equal_to(" | where timestamp < datetime(2021-10-19T22:44:01.100+00:00)".to_owned());
}

#[test]
fn field_filter() {
    let filter = SimpleFieldFilter::new("op".to_owned(), "ze-op".to_owned());
    assert_that(&filter.to_term()).is_equal_to(" | where op == 'ze-op'".to_owned());
}

#[test]
fn build_expression() {
    let operators: Vec<Box<dyn Operator>> = vec![
        Box::new(SimpleFieldFilter::new("foo".to_owned(), "bar".to_owned())),
        Box::new(Ordering::new()),
    ];
    let query = tabular_expression("traces", &operators);
    assert_that(&query)
        .is_equal_to("traces | where foo == 'bar' | sort by timestamp asc".to_owned());
}
