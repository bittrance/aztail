use crate::output::Presenter;
use crate::source::LogSource;
use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset};
use futures::future::join_all;
use itertools::Itertools;

type QuerierArgs = (Vec<Box<dyn LogSource>>, Box<dyn Presenter>, bool);

#[allow(clippy::match_on_vec_items)]
pub async fn querier((mut sources, presenter, follow): QuerierArgs) -> Result<QuerierArgs> {
    let mut source_max_ts = Vec::new();
    source_max_ts.resize(sources.len(), None::<DateTime<FixedOffset>>);
    let streams = join_all(sources.iter().map(|source| source.stream()))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let log_entries = streams
        .into_iter()
        .enumerate()
        .map(|(source_id, stream)| stream.map(move |entry| (source_id, entry)))
        .kmerge_by(|(_, l), (_, r)| l < r);
    for (source_id, log_entry) in log_entries {
        source_max_ts[source_id] = match source_max_ts[source_id] {
            None => Some(log_entry.timestamp()),
            Some(prev_ts) if log_entry.timestamp() > prev_ts => Some(log_entry.timestamp()),
            Some(prev_ts) => Some(prev_ts),
        };
        presenter.present(&log_entry)?;
    }
    if follow {
        for (source_id, max_ts) in source_max_ts.into_iter().enumerate() {
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
    use crate::examples::{T1, T2, T3, T4};
    use crate::kusto::Timespan;
    use crate::testing::*;
    use anyhow::Result;
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
        let source1 = TestSource::with_rows(vec![log_entry(T1), log_entry(T4)]);
        let source2 = TestSource::with_rows(vec![log_entry(T2), log_entry(T3)]);
        let presented = Arc::new(Mutex::new(Vec::new()));
        let presenter = TestPresenter::output_to(&presented);
        querier((vec![source1, source2], presenter, true)).await?;
        let res = Arc::try_unwrap(presented).unwrap().into_inner().unwrap();
        assert_that(&res).has_length(4);
        assert_that(&res[0].timestamp()).is_equal_to(&T1.parse().unwrap());
        assert_that(&res[1].timestamp()).is_equal_to(&T2.parse().unwrap());
        assert_that(&res[2].timestamp()).is_equal_to(&T3.parse().unwrap());
        assert_that(&res[3].timestamp()).is_equal_to(&T4.parse().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn querier_advances_start_time_individually_for_each_stream() -> Result<()> {
        let source1 = TestSource::with_rows(vec![log_entry(T1)]);
        let source2 = TestSource::with_rows(vec![log_entry(T2)]);
        let presenter = TestPresenter::new();
        let (mut sources, _, _) = querier((vec![source1, source2], presenter, true)).await?;
        assert_that(&sources[0].get_query_mut().peek_timespan()).is_equal_to(&Timespan::new(
            "timestamp".to_owned(),
            T1.parse().ok(),
            None,
        ));
        assert_that(&sources[1].get_query_mut().peek_timespan()).is_equal_to(&Timespan::new(
            "timestamp".to_owned(),
            T2.parse().ok(),
            None,
        ));
        Ok(())
    }
}
