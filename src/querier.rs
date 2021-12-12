use crate::appinsights::LogSource;
use crate::output::Presenter;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use futures::future::join_all;
use itertools::Itertools;
use serde_json::value::{Map, Value};

type QuerierArgs = (Vec<Box<dyn LogSource>>, Box<dyn Presenter>, bool);

fn datetime_from_row(row: &Map<String, Value>) -> Option<DateTime<FixedOffset>> {
    row.get("timestamp")
        .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
}

#[allow(clippy::match_on_vec_items)]
pub async fn querier((mut sources, presenter, follow): QuerierArgs) -> Result<QuerierArgs> {
    let mut max_ts_by_stream = Vec::new();
    max_ts_by_stream.resize(sources.len(), None::<DateTime<FixedOffset>>);
    let streams = join_all(sources.iter().map(|source| source.stream()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let log_entries = streams
        .into_iter()
        .enumerate()
        .map(|(source_id, stream)| stream.map(move |entry| (source_id, entry)))
        .kmerge_by(|(_, l), (_, r)| datetime_from_row(l) < datetime_from_row(r));
    for (source_id, row) in log_entries {
        if let Some(ts) = row
            .get("timestamp")
            .map(|v| DateTime::parse_from_rfc3339(v.as_str().unwrap()).unwrap())
        {
            max_ts_by_stream[source_id] = match max_ts_by_stream[source_id] {
                None => Some(ts),
                Some(prev_ts) if ts > prev_ts => Some(ts),
                Some(prev_ts) => Some(prev_ts),
            }
        }
        presenter.present(&row)?;
    }
    if follow {
        for (source_id, max_ts) in max_ts_by_stream.into_iter().enumerate() {
            if max_ts.is_some() {
                sources[source_id].get_query_mut().advance_start(max_ts);
            }
        }
        Ok((sources, presenter, follow))
    } else {
        Err(anyhow!(super::AzTailError::Break))
    }
}

#[cfg(test)]
mod test {
    use super::querier;
    use crate::testing::*;
    use anyhow::Result;
    use serde_json::json;
    use speculoos::prelude::*;
    use std::panic;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn querier_reports_break_when_not_follow() {
        let source = TestSource::with_example_data();
        let presenter = TestPresenter::new();
        match querier((vec![source], presenter, false)).await {
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
        match querier((vec![source], presenter, true)).await {
            Ok(_) => (),
            Err(err) => panic!("Unexpected error {:?}", err),
        };
    }

    #[tokio::test]
    async fn querier_delegates_presentation() {
        let source = TestSource::with_example_data();
        let presented = Arc::new(Mutex::new(Vec::new()));
        let presenter = TestPresenter::output_to(&presented);
        let _ = querier((vec![source], presenter, false)).await;
        let res = Arc::try_unwrap(presented).unwrap().into_inner().unwrap();
        assert_that(&res).has_length(1);
    }

    #[tokio::test]
    async fn querier_sorts_entries_across_streams() -> Result<()> {
        let source1 = TestSource::with_rows(vec![row(T1), row(T4)]);
        let source2 = TestSource::with_rows(vec![row(T2), row(T3)]);
        let presented = Arc::new(Mutex::new(Vec::new()));
        let presenter = TestPresenter::output_to(&presented);
        querier((vec![source1, source2], presenter, true)).await?;
        let res = Arc::try_unwrap(presented).unwrap().into_inner().unwrap();
        assert_that(&res).has_length(4);
        assert_that(&res[0].get("timestamp")).is_equal_to(&Some(&json!(T1)));
        assert_that(&res[1].get("timestamp")).is_equal_to(&Some(&json!(T2)));
        assert_that(&res[2].get("timestamp")).is_equal_to(&Some(&json!(T3)));
        assert_that(&res[3].get("timestamp")).is_equal_to(&Some(&json!(T4)));
        Ok(())
    }

    #[tokio::test]
    async fn querier_advances_start_time_individually_for_each_stream() -> Result<()> {
        let source1 = TestSource::with_rows(vec![row(T1)]);
        let source2 = TestSource::with_rows(vec![row(T2)]);
        let presenter = TestPresenter::new();
        let (mut sources, _, _) = querier((vec![source1, source2], presenter, true)).await?;
        assert_that(&sources[0].get_query_mut().peek_timespan())
            .is_equal_to(Some((T1.parse().ok(), None)));
        assert_that(&sources[1].get_query_mut().peek_timespan())
            .is_equal_to(Some((T2.parse().ok(), None)));
        Ok(())
    }
}
