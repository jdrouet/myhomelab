use anyhow::Context;
use myhomelab_metric::entity::MetricHeader;
use myhomelab_metric::query::{Query, ScalarResponse};
use myhomelab_prelude::time::TimeRange;
use sqlx::types::Json;
use sqlx::{FromRow, Row};

use super::shared::Wrapper;

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<ScalarResponse> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get(0)?;
        Ok(Wrapper(ScalarResponse {
            header: MetricHeader {
                name: name.into(),
                tags: row.try_get(1).map(|Json(inner)| inner)?,
            },
            value: row.try_get(2)?,
        }))
    }
}

pub(super) async fn fetch<'a, E: sqlx::Executor<'a, Database = sqlx::Sqlite>>(
    executor: E,
    query: &Query,
    timerange: &TimeRange,
) -> anyhow::Result<Vec<ScalarResponse>> {
    let mut qb = sqlx::QueryBuilder::<'_, sqlx::Sqlite>::new("with gauge_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, query);
    qb.push(", value");
    qb.push(" from gauge_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange.into_absolute());
    super::shared::build_tags_filter(&mut qb, query.header.iter_tags());
    qb.push("), counter_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, query);
    qb.push(", value");
    qb.push(" from counter_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange.into_absolute());
    super::shared::build_tags_filter(&mut qb, query.header.iter_tags());
    qb.push("), extractions as (");
    qb.push(" select name, tags, value from gauge_extractions");
    qb.push(" union all select name, tags, value from counter_extractions");
    qb.push(") select name, tags");
    super::shared::build_value_attribute(&mut qb, &query.aggregator);
    qb.push(" from extractions");
    qb.push(" group by name, tags");
    let values: Vec<Wrapper<ScalarResponse>> = qb
        .build_query_as()
        .fetch_all(executor)
        .await
        .context("fetching scalar metrics")?;
    Ok(Wrapper::from_many(values))
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::entity::{MetricHeader, MetricTags};
    use myhomelab_metric::query::Query;
    use myhomelab_prelude::time::AbsoluteTimeRange;

    #[tokio::test]
    async fn should_fetch_gauge_max_global() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        // basic
        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max(MetricHeader::new("system.cpu", Default::default())),
            &AbsoluteTimeRange::since(0).into(),
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].value, 90.0);
    }

    #[tokio::test]
    async fn should_fetch_gauge_max_with_group_by() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        // with group by host
        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max(MetricHeader::new("system.cpu", Default::default()))
                .with_group_by(["host"].into_iter()),
            &AbsoluteTimeRange::since(0).into(),
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 2);
    }

    #[tokio::test]
    async fn should_fetch_gauge_min_with_header() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        // basic
        let res = super::fetch(
            sqlite.as_ref(),
            &Query::max(MetricHeader::new(
                "system.cpu",
                MetricTags::default().with_tag("host", "macbook"),
            )),
            &AbsoluteTimeRange::since(0).into(),
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].value, 3.0);
    }

    #[tokio::test]
    async fn should_fetch_counters() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        // basic
        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum(MetricHeader::new("system.reboot", MetricTags::default())),
            &AbsoluteTimeRange::since(0).into(),
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].value, 2.0);
    }

    #[tokio::test]
    async fn scalar_should_return_none_for_missing_metric() {
        let sqlite = crate::query::tests::prepare_pool().await.unwrap();

        let res = super::fetch(
            sqlite.as_ref(),
            &Query::sum(MetricHeader::new("nonexistent.metric", Default::default())),
            &AbsoluteTimeRange::since(0).into(),
        )
        .await
        .unwrap();

        assert_eq!(res.len(), 0);
    }
}
