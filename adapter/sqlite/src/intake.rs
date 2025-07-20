use anyhow::Context;
use myhomelab_metric::{
    entity::{MetricRef, value::MetricValue},
    prelude::MetricFacade,
};

impl myhomelab_metric::intake::Intake for crate::Sqlite {
    async fn ingest<'h>(&self, values: &[MetricRef<'h, MetricValue>]) -> anyhow::Result<()> {
        let mut count: usize = 0;
        let mut builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
            sqlx::QueryBuilder::new("INSERT INTO metrics (name, tags, timestamp, value) ");
        builder.push_values(values.iter(), |mut acc, item| {
            count += 1;
            acc.push_bind(item.name())
                .push_bind(sqlx::types::Json(item.tags()))
                .push_bind(item.timestamp() as i64)
                .push_bind(sqlx::types::Json(item.value()));
        });
        if count > 0 {
            builder
                .build()
                .execute(&self.0)
                .await
                .context("saving metrics")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use myhomelab_metric::entity::value::MetricValue;
    use myhomelab_metric::entity::{MetricHeader, MetricRef};
    use myhomelab_metric::intake::Intake;

    #[tokio::test]
    async fn should_ingest_only_counters() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();
        let header = MetricHeader::new("foo", Default::default());
        sqlite
            .ingest(&[
                MetricRef {
                    header: Cow::Borrowed(&header),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                MetricRef {
                    header: Cow::Borrowed(&header),
                    timestamp: 1,
                    value: MetricValue::counter(41),
                },
                MetricRef {
                    header: Cow::Borrowed(&header),
                    timestamp: 2,
                    value: MetricValue::counter(43),
                },
                MetricRef {
                    header: Cow::Borrowed(&header),
                    timestamp: 3,
                    value: MetricValue::counter(45),
                },
            ])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_ingest_only_gauges() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();
        let foo = MetricHeader::new("foo", Default::default());
        let bar = MetricHeader::new("foo", Default::default());
        sqlite
            .ingest(&[
                MetricRef {
                    header: Cow::Borrowed(&foo),
                    timestamp: 0,
                    value: MetricValue::gauge(1.1),
                },
                MetricRef {
                    header: Cow::Borrowed(&foo),
                    timestamp: 1,
                    value: MetricValue::gauge(1.2),
                },
                MetricRef {
                    header: Cow::Borrowed(&foo),
                    timestamp: 2,
                    value: MetricValue::gauge(-2.0),
                },
                MetricRef {
                    header: Cow::Borrowed(&foo),
                    timestamp: 3,
                    value: MetricValue::gauge(4.1),
                },
                MetricRef {
                    header: Cow::Borrowed(&bar),
                    timestamp: 4,
                    value: MetricValue::gauge(6.1),
                },
            ])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_ingest_multiple_metrics() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();
        sqlite
            .ingest(&[
                MetricRef {
                    header: Cow::Owned(MetricHeader::new("foo", Default::default())),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                MetricRef {
                    header: Cow::Owned(MetricHeader::new("bar", Default::default())),
                    timestamp: 1,
                    value: MetricValue::gauge(42.0),
                },
            ])
            .await
            .unwrap();
    }
}
