use myhomelab_agent_prelude::mpsc::Sender;
use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags};
use myhomelab_prelude::current_timestamp;
use sysinfo::System;
use tokio_util::sync::CancellationToken;

const DEFAULT_INTERVAL: u64 = 10;

#[derive(Debug)]
pub struct ReaderSystemConfig {
    pub enabled: bool,
    pub interval: std::time::Duration,
}

impl Default for ReaderSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: std::time::Duration::new(DEFAULT_INTERVAL, 0),
        }
    }
}

impl myhomelab_prelude::FromEnv for ReaderSystemConfig {
    fn from_env() -> anyhow::Result<Self> {
        let enabled = myhomelab_prelude::parse_from_env("MYHOMELAB_READER_SYSTEM_ENABLED")?;
        let interval: Option<u64> =
            myhomelab_prelude::parse_from_env("MYHOMELAB_READER_SYSTEM_INTERVAL")?;
        Ok(Self {
            enabled: enabled.unwrap_or(true),
            interval: std::time::Duration::new(interval.unwrap_or(DEFAULT_INTERVAL), 0),
        })
    }
}

impl ReaderSystemConfig {
    pub fn build(&self) -> anyhow::Result<Option<ReaderSystem>> {
        if !self.enabled {
            return Ok(None);
        }

        Ok(Some(ReaderSystem {
            interval: tokio::time::interval(self.interval),
            system: sysinfo::System::new_all(),
        }))
    }
}

#[derive(Debug)]
pub struct ReaderSystem {
    interval: tokio::time::Interval,
    system: System,
}

impl ReaderSystem {
    async fn collect_cpu<S: Sender>(
        &self,
        host: &str,
        timestamp: u64,
        sender: &S,
    ) -> anyhow::Result<()> {
        for (index, cpu) in self.system.cpus().iter().enumerate() {
            let tags = MetricTags::default()
                .with_tag("host", host)
                .with_tag("index", index as i64)
                .with_tag("cpu_name", cpu.name())
                .with_tag("cpu_brand", cpu.brand())
                .with_tag("cpu_vendor_id", cpu.vendor_id());
            sender
                .push(Metric {
                    header: MetricHeader::new("system.cpu.frequency", tags.clone()),
                    timestamp,
                    value: MetricValue::gauge(cpu.frequency() as f64),
                })
                .await?;
            sender
                .push(Metric {
                    header: MetricHeader::new("system.cpu.usage", tags),
                    timestamp,
                    value: MetricValue::gauge(cpu.cpu_usage() as f64),
                })
                .await?;
        }
        Ok(())
    }

    async fn collect_memory<S: Sender>(
        &mut self,
        host: &str,
        timestamp: u64,
        sender: &S,
    ) -> anyhow::Result<()> {
        let tags = MetricTags::default().with_tag("host", host);
        sender
            .push(Metric {
                header: MetricHeader::new("system.memory.total", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.total_memory() as f64),
            })
            .await?;
        sender
            .push(Metric {
                header: MetricHeader::new("system.memory.used", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.used_memory() as f64),
            })
            .await?;
        sender
            .push(Metric {
                header: MetricHeader::new("system.swap.total", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.total_swap() as f64),
            })
            .await?;
        sender
            .push(Metric {
                header: MetricHeader::new("system.swap.used", tags.clone()),
                timestamp,
                value: MetricValue::gauge(self.system.used_swap() as f64),
            })
            .await?;
        Ok(())
    }
}

impl myhomelab_agent_prelude::reader::Reader for ReaderSystem {
    #[tracing::instrument(skip_all)]
    async fn run<S: Sender + Send>(
        mut self,
        token: CancellationToken,
        sender: S,
    ) -> anyhow::Result<()> {
        tracing::info!("starting reader");
        let host = System::host_name().unwrap_or_else(|| "unknown".into());
        while !token.is_cancelled() {
            tokio::select! {
                _ = token.cancelled() => {
                    tracing::debug!("cancelled reader");
                }
                _ = self.interval.tick() => {
                    tracing::debug!("collecting metrics");
                    self.system.refresh_all();
                    let now = current_timestamp();
                    self.collect_cpu(&host, now, &sender).await?;
                    self.collect_memory(&host, now, &sender).await?;
                }
            }
        }
        tracing::info!("stopped reader");
        Ok(())
    }
}
