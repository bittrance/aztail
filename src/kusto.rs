use chrono::prelude::*;
use std::any::Any;
use std::fmt::{self, Debug, Display, Formatter};

pub struct Query {
    table: String,
    timespan: Timespan,
    operators: Vec<Box<dyn Operator>>,
}

impl Query {
    pub fn new(table: String, timespan: Timespan, operators: Vec<Box<dyn Operator>>) -> Self {
        Query {
            table,
            timespan,
            operators,
        }
    }

    pub fn advance_start(&mut self, start_time: Option<DateTime<FixedOffset>>) {
        self.timespan.advance_start(start_time);
    }

    #[cfg(test)]
    pub fn peek_timespan<'a>(&'a self) -> &'a Timespan {
        &self.timespan
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.table, self.timespan)?;
        for op in &self.operators {
            write!(f, "{}", op)?;
        }
        Ok(())
    }
}

pub trait Operator: Any + Debug + Send + Sync + Display {}

#[derive(Debug)]
pub struct Filter {
    expression: Box<dyn Expression>,
}

impl Filter {
    pub fn new<T>(expression: T) -> Self
    where
        T: Expression,
    {
        Self {
            expression: Box::new(expression),
        }
    }

    pub fn boxed<T>(expression: T) -> Box<Self>
    where
        T: Expression,
    {
        Box::new(Self::new(expression))
    }
}

impl Operator for Filter {}

impl Display for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, " | where {}", self.expression)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Timespan {
    field: String,
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
}

impl Timespan {
    pub fn new(
        field: String,
        start_time: Option<DateTime<FixedOffset>>,
        end_time: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self {
            field,
            start_time,
            end_time,
        }
    }

    pub fn advance_start(&mut self, start_time: Option<DateTime<FixedOffset>>) {
        self.start_time = start_time;
    }
}

impl Operator for Timespan {}

impl Display for Timespan {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(start_time) = self.start_time {
            let start_filter = Filter::new(Gt::new(self.field.clone(), start_time.to_rfc3339()));
            write!(f, "{}", start_filter)?;
        }
        if let Some(end_time) = self.end_time {
            let end_filter = Filter::new(Lt::new(self.field.clone(), end_time.to_rfc3339()));
            write!(f, "{}", end_filter)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Ordering {
    field: String,
}

impl Ordering {
    pub fn new(field: String) -> Self {
        Self { field }
    }

    pub fn boxed(field: String) -> Box<dyn Operator> {
        Box::new(Self::new(field)) as Box<dyn Operator>
    }
}

impl Operator for Ordering {}

impl Display for Ordering {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, " | sort by {} asc", self.field)
    }
}

pub trait Expression: Any + Debug + Send + Sync + Display {}

#[derive(Debug)]
pub struct Eq {
    field: String,
    value: String,
}

impl Eq {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }

    pub fn boxed(field: String, value: String) -> Box<dyn Expression> {
        Box::new(Self::new(field, value)) as Box<dyn Expression>
    }
}

impl Expression for Eq {}

impl Display for Eq {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} == '{}'", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct StartsWith {
    field: String,
    value: String,
}

impl StartsWith {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }

    pub fn boxed(field: String, value: String) -> Box<dyn Expression> {
        Box::new(Self::new(field, value)) as Box<dyn Expression>
    }
}

impl Expression for StartsWith {}

impl Display for StartsWith {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} startswith_cs '{}'", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct EndsWith {
    field: String,
    value: String,
}

impl EndsWith {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }

    pub fn boxed(field: String, value: String) -> Box<dyn Expression> {
        Box::new(Self::new(field, value)) as Box<dyn Expression>
    }
}

impl Expression for EndsWith {}

impl Display for EndsWith {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} endswith_cs '{}'", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct Lt {
    field: String,
    value: String,
}

impl Lt {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }
}

impl Expression for Lt {}

impl Display for Lt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} < datetime({})", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct Ge {
    field: String,
    value: String,
}

impl Ge {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }
}

impl Expression for Ge {}

impl Display for Ge {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} >= datetime({})", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct Gt {
    field: String,
    value: String,
}

impl Gt {
    pub fn new(field: String, value: String) -> Self {
        Self { field, value }
    }
}

impl Expression for Gt {}

impl Display for Gt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} > datetime({})", self.field, self.value)
    }
}

#[derive(Debug)]
pub struct Or {
    expressions: Vec<Box<dyn Expression>>,
}

impl Or {
    pub fn new(expressions: Vec<Box<dyn Expression>>) -> Self {
        Self { expressions }
    }
}

impl Expression for Or {}

impl Display for Or {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for expr in &self.expressions {
            if !first {
                write!(f, " or ")?;
            }
            first = false;
            write!(f, "{}", expr)?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod test {
    use super::{Eq, Filter, Ge, Gt, Lt, Or, Ordering, Query, Timespan};
    use crate::testing::{T1, T2};
    use speculoos::prelude::*;

    #[test]
    fn empty_query() {
        let query = Query::new(
            "traces".to_owned(),
            Timespan::new("timestamp".to_owned(), None, None),
            vec![Box::new(Ordering::new("timestamp".to_owned()))],
        );
        assert_that(&format!("{}", query)).is_equal_to("traces | sort by timestamp asc".to_owned());
    }

    #[test]
    fn query_with_timespan() {
        let query = Query::new(
            "traces".to_owned(),
            Timespan::new("timestamp".to_owned(), T1.parse().ok(), T2.parse().ok()),
            vec![Box::new(Ordering::new("timestamp".to_owned()))],
        );
        assert_that(&format!("{}", query)).contains("| where timestamp > datetime(");
        assert_that(&format!("{}", query)).contains("| where timestamp < datetime(");
    }

    #[test]
    fn filter() {
        let subject = Filter::new(Eq::new("foo".to_owned(), "bar".to_owned()));
        assert_that(&format!("{}", subject)).is_equal_to(" | where foo == 'bar'".to_owned());
    }

    #[test]
    fn eq() {
        let subject = Eq::new("ze-field".to_owned(), "foo".to_owned());
        assert_that(&format!("{}", subject)).is_equal_to("ze-field == 'foo'".to_owned())
    }

    #[test]
    fn lt() {
        let subject = Lt::new("ze-field".to_owned(), T1.parse().unwrap());
        assert_that(&format!("{}", subject)).is_equal_to(&format!("ze-field < datetime({})", T1));
    }

    #[test]
    fn ge() {
        let subject = Ge::new("ze-field".to_owned(), T1.parse().unwrap());
        assert_that(&format!("{}", subject)).is_equal_to(&format!("ze-field >= datetime({})", T1));
    }

    #[test]
    fn gt() {
        let subject = Gt::new("ze-field".to_owned(), T1.parse().unwrap());
        assert_that(&format!("{}", subject)).is_equal_to(&format!("ze-field > datetime({})", T1));
    }

    #[test]
    fn or() {
        let subject = Or::new(vec![
            Box::new(Eq::new("foo".to_owned(), "bar".to_owned())),
            Box::new(Eq::new("baz".to_owned(), "quux".to_owned())),
        ]);
        assert_that(&format!("{}", subject))
            .is_equal_to(&"(foo == 'bar' or baz == 'quux')".to_owned());
    }
}
