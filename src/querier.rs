use crate::appinsights::LogSource;
use crate::output::Presenter;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};

type QuerierArgs = (Box<dyn LogSource>, Box<dyn Presenter>, bool);

pub async fn querier((mut source, presenter, follow): QuerierArgs) -> Result<QuerierArgs> {
    let mut last_message_ts = None::<DateTime<FixedOffset>>;
    let log_entries = source.stream().await?;
    for row in log_entries {
        if let Some(ts) = row
            .get("timestamp")
            .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
        {
            last_message_ts = match last_message_ts {
                None => Some(ts),
                Some(prev_ts) if ts > prev_ts => Some(ts),
                Some(prev_ts) => Some(prev_ts),
            }
        }
        presenter.present(&row)?;
    }
    if follow {
        if last_message_ts.is_some() {
            source.get_query_mut().advance_start(last_message_ts);
        }
        Ok((source, presenter, follow))
    } else {
        Err(anyhow!(super::AzTailError::Break))
    }
}

#[cfg(test)]
mod test {
    use super::querier;
    use crate::testing::*;
    use speculoos::prelude::*;
    use std::panic;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn querier_reports_break_when_not_follow() {
        let source = TestSource::with_example_data();
        let presenter = TestPresenter::new();
        match querier((Box::new(source), Box::new(presenter), false)).await {
            Ok(_) => panic!("Expected querier to respect follow = false"),
            Err(err) => match err.downcast_ref::<crate::AzTailError>() {
                Some(crate::AzTailError::Break) => (),
                err => panic!("Unexpected error {:?}", err),
            },
        };
    }

    #[tokio::test]
    async fn querier_reports_continue_when_follow() {
        let source = TestSource::with_example_data();
        let presenter = TestPresenter::new();
        match querier((Box::new(source), Box::new(presenter), true)).await {
            Ok(_) => (),
            Err(err) => panic!("Unexpected error {:?}", err),
        };
    }

    #[tokio::test]
    async fn querier_delegates_presentation() {
        let source = TestSource::with_example_data();
        let presented = Arc::new(Mutex::new(Vec::new()));
        let presenter = TestPresenter {
            rows: Arc::clone(&presented),
        };
        let _ = querier((Box::new(source), Box::new(presenter), false)).await;
        let res = Arc::try_unwrap(presented).unwrap().into_inner().unwrap();
        assert_that(&res).has_length(1);
    }

    #[tokio::test]
    async fn querier_advances_start_time() {
        let source = TestSource::with_example_data();
        let presenter = TestPresenter::new();
        match querier((Box::new(source), Box::new(presenter), true)).await {
            Err(err) => panic!("Unexpected error {:?}", err),
            Ok((mut s, _, _)) => assert_that(&s.get_query_mut().peek_timespan())
                .is_equal_to(Some(("2021-11-20T06:18:30+00:00".parse().ok(), None))),
        };
    }
}
