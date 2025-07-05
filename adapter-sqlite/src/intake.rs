use anyhow::Context;
use myhomelab_metric::entity::value::{CounterValue, GaugeValue, MetricValue};
use myhomelab_metric::entity::{Metric, MetricRef};

async fn ingest_counters<'a, E>(
    executor: E,
    values: Vec<MetricRef<'a, CounterValue>>,
) -> anyhow::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    if values.is_empty() {
        return Ok(());
    }

    let mut counter_builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
        sqlx::QueryBuilder::new("INSERT INTO counter_metrics (name, tags, timestamp, value) ");
    counter_builder.push_values(values.into_iter(), |mut acc, item| {
        acc.push_bind(item.header.name.as_ref())
            .push_bind(sqlx::types::Json(&item.header.tags))
            .push_bind(item.timestamp as i64)
            .push_bind(item.value.0 as i64);
    });
    counter_builder
        .build()
        .execute(executor)
        .await
        .context("saving counter metrics")?;
    Ok(())
}

async fn ingest_gauges<'a, E>(
    executor: E,
    values: Vec<MetricRef<'a, GaugeValue>>,
) -> anyhow::Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    if values.is_empty() {
        return Ok(());
    }

    let mut gauge_builder: sqlx::QueryBuilder<'_, sqlx::Sqlite> =
        sqlx::QueryBuilder::new("INSERT INTO gauge_metrics (name, tags, timestamp, value) ");
    gauge_builder.push_values(values.into_iter(), |mut acc, item| {
        acc.push_bind(item.header.name.as_ref())
            .push_bind(sqlx::types::Json(&item.header.tags))
            .push_bind(item.timestamp as i64)
            .push_bind(item.value.0);
    });
    gauge_builder
        .build()
        .execute(executor)
        .await
        .context("saving gauge metrics")?;
    Ok(())
}

impl myhomelab_metric::intake::Intake for crate::Sqlite {
    async fn ingest(&self, values: Vec<Metric>) -> anyhow::Result<()> {
        if values.is_empty() {
            tracing::debug!("empty list of metrics, skipping");
            return Ok(());
        }

        let mut tx = self.0.begin().await.context("opening transaction")?;

        let mut counters = Vec::with_capacity(values.len());
        let mut gauges = Vec::with_capacity(values.len());

        values.iter().for_each(|metric| match metric.value {
            MetricValue::Counter(ref value) => {
                counters.push(MetricRef {
                    header: &metric.header,
                    timestamp: metric.timestamp,
                    value,
                });
            }
            MetricValue::Gauge(ref value) => {
                gauges.push(MetricRef {
                    header: &metric.header,
                    timestamp: metric.timestamp,
                    value,
                });
            }
        });

        ingest_counters(&mut *tx, counters).await?;
        ingest_gauges(&mut *tx, gauges).await?;

        tx.commit().await.context("commiting")?;

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
