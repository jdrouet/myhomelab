use anyhow::Context;
use myhomelab_metric::query::{Query, TimeseriesResponse};
use myhomelab_prelude::time::TimeRange;
use sqlx::types::Json;
use sqlx::{FromRow, Row};

use super::shared::Wrapper;

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<TimeseriesResponse> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get(0)?;
        let timestamps: Json<Vec<u64>> = row.try_get(2)?;
        let values: Json<Vec<f64>> = row.try_get(3)?;
        Ok(Wrapper(TimeseriesResponse {
            name: name.into(),
            tags: row.try_get(1).map(|Json(value)| value)?,
            values: timestamps.0.into_iter().zip(values.0).collect(),
        }))
    }
}

#[tracing::instrument(skip_all, err)]
pub(super) async fn fetch<'a, E: sqlx::Executor<'a, Database = sqlx::Sqlite>>(
    executor: E,
    query: &Query,
    timerange: &TimeRange,
    period: u32,
) -> anyhow::Result<Vec<TimeseriesResponse>> {
    let mut qb = sqlx::QueryBuilder::<'_, sqlx::Sqlite>::new("with extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, query);
    qb.push(", timestamp");
    qb.push(", timestamp / ")
        .push_bind(period)
        .push(" as period");
    qb.push(", timestamp");
    qb.push(", json_extract(value, '$.value') as value");
    qb.push(" from metrics");
    qb.push(" where name = ").push_bind(query.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange.into_absolute());
    super::shared::build_tags_filter(&mut qb, query.tags.iter());
    qb.push("), aggregated_extractions as (");
    qb.push("select name, tags, timestamp");
    super::shared::build_value_attribute(&mut qb, &query.aggregator);
    qb.push(" from extractions");
    qb.push(" group by name, tags, period");
    qb.push(") select name, tags, json_group_array(timestamp), json_group_array(value)");
    qb.push(" from aggregated_extractions");
    qb.push(" group by name, tags");
    let values: Vec<Wrapper<TimeseriesResponse>> = qb
        .build_query_as()
        .fetch_all(executor)
        .await
        .context("fetching timeseries metrics")?;
    Ok(Wrapper::from_many(values))
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::entity::MetricTags;
    use myhomelab_metric::entity::tag::TagValue;
    use myhomelab_metric::query::Query;
    use myhomelab_prelude::time::AbsoluteTimeRange;

    #[tokio::test]
    async fn should_fetch_gauge_max_global() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max("system.cpu", Default::default()),
            &AbsoluteTimeRange::since(0).into(),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.name.as_ref(), "system.cpu");
        assert!(!entry.values.is_empty());
    }

    #[tokio::test]
    async fn should_fetch_gauge_max_with_group_by() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max("system.cpu", Default::default()).with_group_by(["host"].into_iter()),
            &AbsoluteTimeRange::since(0).into(),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 2);
        for entry in &res {
            assert_eq!(entry.name.as_ref(), "system.cpu");
            assert!(entry.tags.get("host").is_some());
        }
    }

    #[tokio::test]
    async fn should_fetch_gauge_min_with_header() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::min(
                "system.cpu",
                MetricTags::default().with_tag("host", "macbook"),
            ),
            &AbsoluteTimeRange::since(0).into(),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.name.as_ref(), "system.cpu");
        assert_eq!(
            entry.tags.get("host").unwrap(),
            &TagValue::Text("macbook".into())
        );
    }

    #[tokio::test]
    async fn should_fetch_counters() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum("system.reboot", MetricTags::default()),
            &AbsoluteTimeRange::since(0).into(),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.name.as_ref(), "system.reboot");
        assert!(!entry.values.is_empty());
    }

    #[tokio::test]
    async fn should_return_none_for_missing_metric() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum("nonexistent.metric", Default::default()),
            &AbsoluteTimeRange::since(0).into(),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 0);
    }
}
