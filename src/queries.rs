use chrono::prelude::*;

#[derive(Clone, Debug)]
pub struct QueryParams {
    pub item_type: String,
    pub start_time: Option<DateTime<FixedOffset>>,
    pub end_time: Option<DateTime<FixedOffset>>,
}

pub fn build_query(params: &QueryParams) -> String {
    let mut query = params.item_type.clone();
    if let Some(start_time) = params.start_time {
        query.push_str(&format!(" | where timestamp > datetime({:?})", start_time));
    }
    query.push_str(" | sort by timestamp asc");
    return query;
}

#[test]
fn test_basic_query() {
    let params = QueryParams {
        item_type: "traces".to_string(),
        start_time: "2021-10-19T21:44:01.10Z".parse().ok(),
        end_time: None,
    };
    let query = build_query(&params);
    assert_eq!(
        query,
        "traces | where timestamp > datetime(2021-10-19T21:44:01.100+00:00) | sort by timestamp asc"
    );
}
