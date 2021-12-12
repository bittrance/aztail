use chrono::prelude::*;
use std::any::Any;
use std::fmt::Debug;

pub struct Query {
    table: String,
    timespan_pos: Option<usize>,
    operators: Vec<Box<dyn Operator>>,
}

impl Query {
    pub fn new(table: String, operators: Vec<Box<dyn Operator>>) -> Self {
        let timespan_pos = operators
            .iter()
            .position(|o| o.as_any().is::<TimespanFilter>());
        Query {
            table,
            timespan_pos,
            operators,
        }
    }

    pub fn tabular_expression(&self) -> String {
        let mut expr = self.table.clone();
        expr.push_str(
            &self
                .operators
                .iter()
                .map(|o| o.to_term())
                .collect::<String>(),
        );
        expr
    }

    pub fn advance_start(&mut self, start_time: Option<DateTime<FixedOffset>>) {
        let pos = self.timespan_pos.expect("No timespan filter in query");
        self.operators[pos]
            .as_any_mut()
            .downcast_mut::<TimespanFilter>()
            .unwrap()
            .advance_start(start_time);
    }

    #[cfg(test)]
    pub fn peek_timespan(
        &self,
    ) -> Option<(Option<DateTime<FixedOffset>>, Option<DateTime<FixedOffset>>)> {
        self.timespan_pos
            .and_then(|p| self.operators[p].as_any().downcast_ref::<TimespanFilter>())
            .map(|filter| (filter.start_time, filter.end_time))
    }
}

pub trait Operator: Any + Debug + Send + Sync {
    fn to_term(&self) -> String;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Ordering;

impl Operator for Ordering {
    fn to_term(&self) -> String {
        " | sort by timestamp asc".to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod test {
    use super::{Operator, Ordering, Query, SimpleFieldFilter, TimespanFilter};
    use speculoos::prelude::*;

    #[test]
    fn basic_query_tabular_expression() {
        let operators: Vec<Box<dyn Operator>> = vec![
            Box::new(SimpleFieldFilter::new("foo".to_owned(), "bar".to_owned())),
            Box::new(Ordering),
        ];
        let query = Query::new("traces".to_owned(), operators);
        assert_that(&query.tabular_expression())
            .is_equal_to("traces | where foo == 'bar' | sort by timestamp asc".to_owned());
    }

    #[test]
    fn query_advance_start_time() {
        let operators: Vec<Box<dyn Operator>> = vec![Box::new(TimespanFilter::new(
            "2021-10-19T21:44:01.10Z".parse().ok(),
            None,
        ))];
        let mut query = Query::new("traces".to_owned(), operators);
        query.advance_start("2021-10-19T21:45:01.99Z".parse().ok());
        assert_that(&query.tabular_expression()).is_equal_to(
            "traces | where timestamp > datetime(2021-10-19T21:45:01.990+00:00)".to_owned(),
        );
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
}
