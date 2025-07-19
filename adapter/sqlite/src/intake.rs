use anyhow::Context;
use myhomelab_metric::entity::Metric;

impl myhomelab_metric::intake::Intake for crate::Sqlite {
    async fn ingest(&self, values: Vec<Metric>) -> anyhow::Result<()> {
        if values.is_empty() {
            tracing::debug!("empty list of metrics, skipping");
            return Ok(());
        }

        let mut gauge_builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
            sqlx::QueryBuilder::new("INSERT INTO metrics (name, tags, timestamp, value) ");
        gauge_builder.push_values(values.into_iter(), |mut acc, item| {
            acc.push_bind(item.header.name)
                .push_bind(sqlx::types::Json(item.header.tags))
                .push_bind(item.timestamp as i64)
                .push_bind(sqlx::types::Json(item.value));
        });
        gauge_builder
            .build()
            .execute(&self.0)
            .await
            .context("saving metrics")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use myhomelab_metric::entity::value::MetricValue;
    use myhomelab_metric::entity::{Metric, MetricHeader};
    use myhomelab_metric::intake::Intake;

    #[tokio::test]
    async fn should_ingest_only_counters() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();
        sqlite
            .ingest(vec![
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 1,
                    value: MetricValue::counter(41),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 2,
                    value: MetricValue::counter(43),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
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
            .ingest(vec![
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 0,
                    value: MetricValue::gauge(1.1),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 1,
                    value: MetricValue::gauge(1.2),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 2,
                    value: MetricValue::gauge(-2.0),
                },
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 3,
                    value: MetricValue::gauge(4.1),
                },
                Metric {
                    header: MetricHeader::new("bar", Default::default()),
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
            .ingest(vec![
                Metric {
                    header: MetricHeader::new("foo", Default::default()),
                    timestamp: 0,
                    value: MetricValue::counter(42),
                },
                Metric {
                    header: MetricHeader::new("bar", Default::default()),
                    timestamp: 1,
                    value: MetricValue::gauge(42.0),
                },
            ])
            .await
            .unwrap();
    }
}
