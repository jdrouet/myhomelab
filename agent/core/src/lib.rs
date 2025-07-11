use std::time::Duration;

use myhomelab_agent_prelude::reader::Reader;
use myhomelab_metric::entity::Metric;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ManagerConfig {
    pub buffer_max_size: usize,
    pub interval: Duration,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            buffer_max_size: 200,
            interval: std::time::Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
pub struct ManagerBuilder<I, S, R> {
    cancel: CancellationToken,
    intake: I,
    tasks: Vec<tokio::task::JoinHandle<anyhow::Result<()>>>,
    sender: S,
    receiver: R,
}

impl<I, S, R> ManagerBuilder<I, S, R>
where
    I: myhomelab_metric::intake::Intake,
    S: myhomelab_agent_prelude::mpsc::Sender,
    R: myhomelab_agent_prelude::mpsc::Receiver,
{
    pub fn new(cancel: CancellationToken, intake: I, sender: S, receiver: R) -> Self {
        Self {
            cancel,
            intake,
            sender,
            receiver,
            tasks: Vec::default(),
        }
    }

    pub fn with_reader<E: Reader>(mut self, reader: E) -> Self {
        self.add_reader(reader);
        self
    }

    pub fn add_reader<E: Reader>(&mut self, reader: E) {
        let cancel = self.cancel.child_token();
        let sender = self.sender.clone();
        self.tasks.push(tokio::task::spawn(async move {
            reader.run(cancel, sender).await
        }));
    }

    pub fn build(self, config: &ManagerConfig) -> Manager<I, R> {
        Manager {
            cancel: self.cancel,
            buffer: Vec::with_capacity(config.buffer_max_size),
            buffer_max_size: config.buffer_max_size,
            intake: self.intake,
            interval: tokio::time::interval(config.interval),
            receiver: self.receiver,
            tasks: self.tasks,
        }
    }
}

#[derive(Debug)]
pub struct Manager<I, R> {
    buffer: Vec<Metric>,
    buffer_max_size: usize,
    intake: I,
    interval: tokio::time::Interval,
    receiver: R,
    cancel: CancellationToken,
    #[allow(unused)]
    tasks: Vec<tokio::task::JoinHandle<anyhow::Result<()>>>,
}

impl<I: myhomelab_metric::intake::Intake> Manager<I, tokio::sync::mpsc::UnboundedReceiver<Metric>> {
    pub fn unbounded_builder(
        cancel: CancellationToken,
        intake: I,
    ) -> ManagerBuilder<
        I,
        tokio::sync::mpsc::UnboundedSender<Metric>,
        tokio::sync::mpsc::UnboundedReceiver<Metric>,
    > {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        ManagerBuilder::new(cancel, intake, sender, receiver)
    }
}

impl<I, R> Manager<I, R>
where
    I: myhomelab_metric::intake::Intake,
    R: myhomelab_agent_prelude::mpsc::Receiver,
{
    async fn flush(&mut self) -> anyhow::Result<()> {
        tracing::info!(message = "flushing metrics", count = self.buffer.len());
        self.intake.ingest(std::mem::take(&mut self.buffer)).await
    }

    async fn handle_metric(&mut self, metric: Metric) -> anyhow::Result<()> {
        self.buffer.push(metric);
        if self.buffer.len() >= self.buffer_max_size {
            self.flush().await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("starting manager");
        while !(self.cancel.is_cancelled() && self.receiver.is_empty()) {
            tokio::select! {
                maybe_metric = self.receiver.pull() => {
                    match maybe_metric {
                        Some(item) => {
                            self.handle_metric(item).await?;
                            self.interval.reset();
                        },
                        None => {
                            if !self.buffer.is_empty() {
                                self.flush().await?;
                            }
                            break;
                        }
                    };
                }
                _ = self.interval.tick() => {
                    tracing::debug!("ticking");
                    if !self.buffer.is_empty() {
                        self.flush().await?;
                    }
                }
                _ = self.cancel.cancelled() => {
                    tracing::debug!("cancelled");
                }
            }
        }
        tracing::debug!("stopping tasks");
        while let Some(task) = self.tasks.pop() {
            if let Err(err) = task.await {
                tracing::error!(message = "reader failed", cause = %err);
            }
        }
        tracing::info!("stopped manager");
        Ok(())
    }
}
