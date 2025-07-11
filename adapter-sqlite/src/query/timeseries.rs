use anyhow::Context;
use myhomelab_metric::entity::MetricHeader;
use myhomelab_metric::query::{Query, TimeRange, TimeseriesResponse};
use sqlx::types::Json;
use sqlx::{FromRow, Row};

use super::shared::Wrapper;

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<TimeseriesResponse> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get(0)?;
        let timestamps: Json<Vec<i64>> = row.try_get(2)?;
        let values: Json<Vec<f64>> = row.try_get(3)?;
        Ok(Wrapper(TimeseriesResponse {
            header: MetricHeader {
                name: name.into(),
                tags: row.try_get(1).map(|Json(value)| value)?,
            },
            values: timestamps.0.into_iter().zip(values.0).collect(),
        }))
    }
}

pub(super) async fn fetch<'a, E: sqlx::Executor<'a, Database = sqlx::Sqlite>>(
    executor: E,
    query: &Query,
    timerange: &TimeRange,
    period: u32,
) -> anyhow::Result<Vec<TimeseriesResponse>> {
    let mut qb = sqlx::QueryBuilder::<'_, sqlx::Sqlite>::new("with gauge_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, query);
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
    super::shared::build_tags_filter(&mut qb, query.header.iter_tags());
    qb.push("), counter_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, query);
    qb.push(", timestamp");
    qb.push(", timestamp / ")
        .push_bind(period)
        .push(" as period");
    qb.push(", value");
    qb.push(" from counter_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange);
    super::shared::build_tags_filter(&mut qb, query.header.iter_tags());
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
    let values: Vec<Wrapper<TimeseriesResponse>> = qb
        .build_query_as()
        .fetch_all(executor)
        .await
        .context("fetching timeseries metrics")?;
    Ok(Wrapper::from_many(values))
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::entity::tag::TagValue;
    use myhomelab_metric::entity::{MetricHeader, MetricTags};
    use myhomelab_metric::query::{Query, TimeRange};

    #[tokio::test]
    async fn should_fetch_gauge_max_global() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max(MetricHeader::new("system.cpu", Default::default())),
            &TimeRange::from(0),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.header.name.as_ref(), "system.cpu");
        assert!(!entry.values.is_empty());
    }

    #[tokio::test]
    async fn should_fetch_gauge_max_with_group_by() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max(MetricHeader::new("system.cpu", Default::default()))
                .with_group_by(["host"].into_iter()),
            &TimeRange::from(0),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 2);
        for entry in &res {
            assert_eq!(entry.header.name.as_ref(), "system.cpu");
            assert!(entry.header.tag("host").is_some());
        }
    }

    #[tokio::test]
    async fn should_fetch_gauge_min_with_header() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::min(MetricHeader::new(
                "system.cpu",
                MetricTags::default().with_tag("host", "macbook"),
            )),
            &TimeRange::from(0),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.header.name.as_ref(), "system.cpu");
        assert_eq!(
            entry.header.tag("host").unwrap(),
            &TagValue::Text("macbook".into())
        );
    }

    #[tokio::test]
    async fn should_fetch_counters() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum(MetricHeader::new("system.reboot", MetricTags::default())),
            &TimeRange::from(0),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        let entry = &res[0];
        assert_eq!(entry.header.name.as_ref(), "system.reboot");
        assert!(!entry.values.is_empty());
    }

    #[tokio::test]
    async fn should_return_none_for_missing_metric() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum(MetricHeader::new("nonexistent.metric", Default::default())),
            &TimeRange::from(0),
            3,
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 0);
    }
}
