use anyhow::Context;
use myhomelab_metric::entity::Metric;
use myhomelab_metric::entity::value::MetricValue;

impl myhomelab_metric::intake::Intake for crate::Sqlite {
    #[tracing::instrument(skip_all, err)]
    async fn ingest<'h>(&self, values: &[Metric<'h, MetricValue>]) -> anyhow::Result<()> {
        let mut count: usize = 0;
        let mut builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
            sqlx::QueryBuilder::new("INSERT INTO metrics (name, tags, timestamp, value) ");
        builder.push_values(values.iter(), |mut acc, item| {
            count += 1;
            acc.push_bind(&item.name)
                .push_bind(sqlx::types::Json(&item.tags))
                .push_bind(item.timestamp as i64)
                .push_bind(sqlx::types::Json(&item.value));
        });
        if count > 0 {
            builder
                .build()
                .execute(&self.pool)
                .await
                .context("saving metrics")?;
            tracing::debug!(message = "inserted metrics", count);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_metric::entity::Metric;
    use myhomelab_metric::entity::value::MetricValue;
    use myhomelab_metric::intake::Intake;

    #[tokio::test]
    async fn should_ingest_only_counters() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();
        sqlite
            .ingest(&[
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 1,
                    value: MetricValue::counter(41),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 2,
                    value: MetricValue::counter(43),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
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
        sqlite
            .ingest(&[
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 0,
                    value: MetricValue::gauge(1.1),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 1,
                    value: MetricValue::gauge(1.2),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 2,
                    value: MetricValue::gauge(-2.0),
                },
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 3,
                    value: MetricValue::gauge(4.1),
                },
                Metric {
                    name: "bar".into(),
                    tags: Default::default(),
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
                Metric {
                    name: "foo".into(),
                    tags: Default::default(),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                Metric {
                    name: "bar".into(),
                    tags: Default::default(),
                    timestamp: 1,
                    value: MetricValue::gauge(42.0),
                },
            ])
            .await
            .unwrap();
    }
}
