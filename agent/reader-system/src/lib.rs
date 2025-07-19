use std::time::Duration;

use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_agent_prelude::reader::{BasicTaskReader, BuildContext};
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags};
use myhomelab_prelude::time::current_timestamp;
use sysinfo::System;
use tokio_util::sync::CancellationToken;

#[derive(Debug, serde::Deserialize)]
pub struct SystemReaderConfig {
    #[serde(default = "SystemReaderConfig::default_interval")]
    pub interval: u64,
}

impl Default for SystemReaderConfig {
    fn default() -> Self {
        Self {
            interval: Self::default_interval(),
        }
    }
}

impl SystemReaderConfig {
    const fn default_interval() -> u64 {
        10_000
    }
}

impl myhomelab_agent_prelude::reader::ReaderBuilder for SystemReaderConfig {
    type Output = SystemReader;

    async fn build<S: Sender>(&self, ctx: &BuildContext<S>) -> anyhow::Result<Self::Output> {
        let runner = SystemRunner {
            cancel: ctx.cancel.child_token(),
            interval: tokio::time::interval(Duration::from_millis(self.interval)),
            sender: ctx.sender.clone(),
            system: sysinfo::System::new_all(),
        };
        let task = tokio::task::spawn(async move { runner.run().await });
        Ok(BasicTaskReader::new(task))
    }
}

struct SystemRunner<S> {
    cancel: CancellationToken,
    interval: tokio::time::Interval,
    system: System,
    sender: S,
}

impl<S: Sender> SystemRunner<S> {
    async fn collect_cpu(&self, host: &str, timestamp: u64) -> anyhow::Result<()> {
        for (index, cpu) in self.system.cpus().iter().enumerate() {
            let tags = MetricTags::default()
                .with_tag("host", host)
                .with_tag("index", index as i64)
                .with_tag("cpu_name", cpu.name())
                .with_tag("cpu_brand", cpu.brand())
                .with_tag("cpu_vendor_id", cpu.vendor_id());
            self.sender
                .push(Metric {
                    header: MetricHeader::new("system.cpu.frequency", tags.clone()),
                    timestamp,
                    value: MetricValue::gauge(cpu.frequency() as f64),
                })
                .await?;
            self.sender
                .push(Metric {
                    header: MetricHeader::new("system.cpu.usage", tags),
                    timestamp,
                    value: MetricValue::gauge(cpu.cpu_usage() as f64),
                })
                .await?;
        }
        Ok(())
    }

    async fn collect_memory(&mut self, host: &str, timestamp: u64) -> anyhow::Result<()> {
        let tags = MetricTags::default().with_tag("host", host);
        self.sender
            .push(Metric {
                header: MetricHeader::new("system.memory.total", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.total_memory() as f64),
            })
            .await?;
        self.sender
            .push(Metric {
                header: MetricHeader::new("system.memory.used", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.used_memory() as f64),
            })
            .await?;
        self.sender
            .push(Metric {
                header: MetricHeader::new("system.swap.total", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.total_swap() as f64),
            })
            .await?;
        self.sender
            .push(Metric {
                header: MetricHeader::new("system.swap.used", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.used_swap() as f64),
            })
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

pub type SystemReader = BasicTaskReader;
