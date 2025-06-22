use super::shared::Wrapper;
use anyhow::Context;
use myhomelab_metric::metric::MetricHeader;
use myhomelab_metric::query::{Query, QueryResponse, ScalarQueryResponse, TimeRange};
use sqlx::{FromRow, Row, types::Json};

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Wrapper<ScalarQueryResponse> {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Wrapper(ScalarQueryResponse {
            header: MetricHeader {
                name: row.try_get(0)?,
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
) -> anyhow::Result<QueryResponse> {
    let mut qb = sqlx::QueryBuilder::<'_, sqlx::Sqlite>::new("with gauge_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, &query);
    qb.push(", value");
    qb.push(" from gauge_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange);
    super::shared::build_tags_filter(&mut qb, query.header.tags.iter());
    qb.push("), counter_extractions as (");
    qb.push("select name");
    super::shared::build_tags_attribute(&mut qb, &query);
    qb.push(", value");
    qb.push(" from counter_metrics");
    qb.push(" where name = ")
        .push_bind(query.header.name.as_ref());
    super::shared::build_timerange_filter(&mut qb, timerange);
    super::shared::build_tags_filter(&mut qb, query.header.tags.iter());
    qb.push("), extractions as (");
    qb.push(" select name, tags, value from gauge_extractions");
    qb.push(" union all select name, tags, value from counter_extractions");
    qb.push(") select name, tags");
    super::shared::build_value_attribute(&mut qb, &query.aggregator);
    qb.push(" from extractions");
    qb.push(" group by name, tags");
    let values: Vec<Wrapper<ScalarQueryResponse>> = qb
        .build_query_as()
        .fetch_all(executor)
        .await
        .context("fetching scalar metrics")?;
    let values = Wrapper::from_many(values);
    Ok(QueryResponse::Scalar(values))
}

#[cfg(test)]
pub(crate) mod tests {
    use myhomelab_metric::{
        metric::{MetricHeader, tag::TagValue},
        query::{Query, QueryExecutor, QueryResponse, Request, RequestKind, TimeRange},
    };

    #[tokio::test]
    async fn should_fetch_simple_scalar() -> anyhow::Result<()> {
        let sqlite = crate::query::tests::prepare_pool().await?;

        let res = sqlite
            .execute(
                &[Request::scalar()
                    .with_query("max-cpu", Query::max(MetricHeader::new("system.cpu")))
                    .with_query("min-cpu", Query::min(MetricHeader::new("system.cpu")))],
                TimeRange::from(0),
            )
            .await?;

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].kind, RequestKind::Scalar);
        assert_eq!(res[0].queries.len(), 2);
        let qres = &res[0].queries["max-cpu"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, 90.0);

        let qres = &res[0].queries["min-cpu"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn should_fetch_simple_scalar_with_headers() -> anyhow::Result<()> {
        let sqlite = crate::query::tests::prepare_pool().await?;

        let res = sqlite
            .execute(
                &[Request::scalar()
                    .with_query(
                        "max-cpu-fr",
                        Query::max(MetricHeader::new("system.cpu").with_tag("location", "FR")),
                    )
                    .with_query(
                        "max-cpu-es",
                        Query::min(MetricHeader::new("system.cpu").with_tag("location", "ES")),
                    )],
                TimeRange::from(0),
            )
            .await?;

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].kind, RequestKind::Scalar);
        assert_eq!(res[0].queries.len(), 2);
        let qres = &res[0].queries["max-cpu-fr"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, 90.0);

        let qres = &res[0].queries["max-cpu-es"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, 10.0);

        Ok(())
    }

    #[tokio::test]
    async fn should_fetch_counters_sum() -> anyhow::Result<()> {
        let sqlite = crate::query::tests::prepare_pool().await?;

        let res = sqlite
            .execute(
                &[Request::scalar()
                    .with_query("reboot-all", Query::sum(MetricHeader::new("system.reboot")))
                    .with_query(
                        "reboot-macbook",
                        Query::sum(MetricHeader::new("system.reboot").with_tag("host", "macbook")),
                    )
                    .with_query(
                        "reboot-raspberry",
                        Query::sum(
                            MetricHeader::new("system.reboot").with_tag("host", "raspberry"),
                        ),
                    )],
                TimeRange::from(0),
            )
            .await?;

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].kind, RequestKind::Scalar);
        assert_eq!(res[0].queries.len(), 3);
        let qres = &res[0].queries["reboot-all"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, 2.0);

        let qres = &res[0].queries["reboot-macbook"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].header.name.as_ref(), "system.reboot");
        assert_eq!(entries[0].header.tags["host"], TagValue::from("macbook"));
        assert_eq!(entries[0].value, 2.0);

        let qres = &res[0].queries["reboot-raspberry"];
        let QueryResponse::Scalar(entries) = qres else {
            panic!("should be a scalar response");
        };
        assert!(
            entries.is_empty(),
            "there's no reboot entry for the raspberry host"
        );

        Ok(())
    }
}
