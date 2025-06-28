use anyhow::Context;
use myhomelab_metric::entity::MetricHeader;
use myhomelab_metric::query::{Query, QueryResponse, TimeRange, TimeseriesQueryResponse};
use sqlx::types::Json;
use sqlx::{FromRow, Row};

use super::shared::Wrapper;

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<TimeseriesQueryResponse> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let timestamps: Json<Vec<i64>> = row.try_get(2)?;
        let values: Json<Vec<f64>> = row.try_get(3)?;
        Ok(Wrapper(TimeseriesQueryResponse {
            header: MetricHeader {
                name: row.try_get(0)?,
                tags: row.try_get(1).map(|Json(value)| value)?,
            },
            values: timestamps.0.into_iter().zip(values.0.into_iter()).collect(),
        }))
    }
}

pub(super) async fn fetch<'a, E: sqlx::Executor<'a, Database = sqlx::Sqlite>>(
    executor: E,
    query: &Query,
    timerange: &TimeRange,
    period: u32,
) -> anyhow::Result<QueryResponse> {
    let mut qb = sqlx::QueryBuilder::<'_, sqlx::Sqlite>::new("with gauge_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, &query);
    qb.push(", timestamp");
    qb.push(", timestamp / ")
        .push_bind(period)
        .push(" as period");
    qb.push(", timestamp");
    qb.push(", value");
    qb.push(" from gauge_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange);
    super::shared::build_tags_filter(&mut qb, query.header.tags.iter());
    qb.push("), counter_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, &query);
    qb.push(", timestamp");
    qb.push(", timestamp / ")
        .push_bind(period)
        .push(" as period");
    qb.push(", value");
    qb.push(" from counter_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange);
    super::shared::build_tags_filter(&mut qb, query.header.tags.iter());
    qb.push("), join_extractions as (");
    qb.push(" select name, tags, timestamp, period, value from gauge_extractions");
    qb.push(" union all select name, tags, timestamp, period, value from counter_extractions");
    qb.push("), aggregated_extractions as (");
    qb.push("select name, tags, timestamp");
    super::shared::build_value_attribute(&mut qb, &query.aggregator);
    qb.push(" from join_extractions");
    qb.push(" group by name, tags, period");
    qb.push(") select name, tags, json_group_array(timestamp), json_group_array(value)");
    qb.push(" from aggregated_extractions");
    qb.push(" group by name, tags");
    let values: Vec<Wrapper<TimeseriesQueryResponse>> = qb
        .build_query_as()
        .fetch_all(executor)
        .await
        .context("fetching timeseries metrics")?;
    let values = Wrapper::from_many(values);
    Ok(QueryResponse::Timeseries(values))
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::entity::MetricHeader;
    use myhomelab_metric::query::{
        Query, QueryExecutor, QueryResponse, Request, RequestKind, TimeRange,
    };

    #[tokio::test]
    async fn should_fetch() -> anyhow::Result<()> {
        let sqlite = crate::query::tests::prepare_pool().await?;

        let res = sqlite
            .execute(
                vec![
                    Request::timeseries(3)
                        .with_query("cpu", Query::max(MetricHeader::new("system.cpu")))
                        .with_query(
                            "cpu-raspberry",
                            Query::min(
                                MetricHeader::new("system.cpu").with_tag("host", "raspberry"),
                            ),
                        ),
                ],
                TimeRange::from(0),
            )
            .await?;

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].kind, RequestKind::Timeseries { period: 3 });
        assert_eq!(res[0].queries.len(), 2);
        let qres = &res[0].queries["cpu"];
        let QueryResponse::Timeseries(entries) = qres else {
            panic!("should be a timeseries response");
        };
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.header.name.as_ref(), "system.cpu");
        assert!(entry.header.tags.is_empty());
        assert_eq!(entry.values, vec![(2, 90.0), (3, 50.0)]);

        let qres = &res[0].queries["cpu-raspberry"];
        let QueryResponse::Timeseries(entries) = qres else {
            panic!("should be a timeseries response");
        };
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.header.name.as_ref(), "system.cpu");
        assert!(entry.header.tags.contains_key("host"));
        assert_eq!(entry.values, vec![(1, 10.0), (4, 20.0)]);

        Ok(())
    }
}
