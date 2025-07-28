use std::borrow::Cow;
use std::time::Duration;

use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricTags};
use myhomelab_prelude::time::current_timestamp;
use myhomelab_sensor_prelude::collector::Collector;
use myhomelab_sensor_prelude::sensor::{BasicTaskSensor, BuildContext};
use sysinfo::System;
use tokio_util::sync::CancellationToken;

#[derive(Debug, serde::Deserialize)]
pub struct SystemSensorConfig {
    #[serde(default = "SystemSensorConfig::default_interval")]
    pub interval: u64,
}

impl Default for SystemSensorConfig {
    fn default() -> Self {
        Self {
            interval: Self::default_interval(),
        }
    }
}

impl SystemSensorConfig {
    const fn default_interval() -> u64 {
        10_000
    }
}

impl myhomelab_sensor_prelude::sensor::SensorBuilder for SystemSensorConfig {
    type Output = SystemSensor;

    async fn build<C: Collector>(&self, ctx: &BuildContext<C>) -> anyhow::Result<Self::Output> {
        let runner = SystemRunner {
            cancel: ctx.cancel.child_token(),
            collector: ctx.collector.clone(),
            interval: tokio::time::interval(Duration::from_millis(self.interval)),
            system: sysinfo::System::new_all(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });
        Ok(BasicTaskSensor::new("system", task))
    }
}

struct SystemRunner<C> {
    cancel: CancellationToken,
    collector: C,
    interval: tokio::time::Interval,
    system: System,
}

impl<C: Collector> SystemRunner<C> {
    async fn collect_cpu(&self, host: &str, timestamp: u64) -> anyhow::Result<()> {
        for (index, cpu) in self.system.cpus().iter().enumerate() {
            let tags = MetricTags::default()
                .with_tag("host", host)
                .with_tag("index", index as i64)
                .with_tag("cpu_name", cpu.name())
                .with_tag("cpu_brand", cpu.brand())
                .with_tag("cpu_vendor_id", cpu.vendor_id());
            self.collector
                .push_metrics(&[
                    Metric {
                        name: "system.cpu.frequency".into(),
                        tags: Cow::Borrowed(&tags),
                        timestamp,
                        value: MetricValue::gauge(cpu.frequency() as f64),
                    },
                    Metric {
                        name: "system.cpu.usage".into(),
                        tags: Cow::Borrowed(&tags),
                        timestamp,
                        value: MetricValue::gauge(cpu.cpu_usage() as f64),
                    },
                ])
                .await?;
        }
        Ok(())
    }

    async fn collect_memory(&mut self, host: &str, timestamp: u64) -> anyhow::Result<()> {
        let tags = MetricTags::default().with_tag("host", host);
        self.collector
            .push_metrics(&[
                Metric {
                    name: "system.memory.total".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp,
                    value: MetricValue::gauge(self.system.total_memory() as f64),
                },
                Metric {
                    name: "system.memory.used".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp,
                    value: MetricValue::gauge(self.system.used_memory() as f64),
                },
                Metric {
                    name: "system.swap.total".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp,
                    value: MetricValue::gauge(self.system.total_swap() as f64),
                },
                Metric {
                    name: "system.swap.used".into(),
                    tags: Cow::Borrowed(&tags),
                    timestamp,
                    value: MetricValue::gauge(self.system.used_swap() as f64),
                },
            ])
            .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, host), err)]
    async fn collect(&mut self, host: &str) -> anyhow::Result<()> {
        self.system.refresh_all();
        let now = current_timestamp();
        self.collect_cpu(host, now).await?;
        self.collect_memory(host, now).await?;
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting");
        let host = System::host_name().unwrap_or_else(|| "unknown".into());
        while !self.cancel.is_cancelled() {
            tokio::select! {
                _ = self.interval.tick() => {
                    let _ = self.collect(&host).await;
                }
                _ = self.cancel.cancelled() => {
                    tracing::debug!("cancellation requested");
                }
            }
        }
        tracing::info!("completed");
        Ok(())
    }
}

pub type SystemSensor = BasicTaskSensor;
