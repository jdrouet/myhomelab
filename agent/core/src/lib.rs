use std::time::Duration;

use myhomelab_agent_prelude::reader::Reader;
use myhomelab_metric::entity::Metric;

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
    pub fn new(intake: I, sender: S, receiver: R) -> Self {
        Self {
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
        let sender = self.sender.clone();
        self.tasks
            .push(tokio::task::spawn(async move { reader.run(sender).await }));
    }

    pub fn build(self, config: &ManagerConfig) -> Manager<I, R> {
        Manager {
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
    #[allow(unused)]
    tasks: Vec<tokio::task::JoinHandle<anyhow::Result<()>>>,
}

impl<I: myhomelab_metric::intake::Intake> Manager<I, tokio::sync::mpsc::UnboundedReceiver<Metric>> {
    pub fn unbounded_builder(
        intake: I,
    ) -> ManagerBuilder<
        I,
        tokio::sync::mpsc::UnboundedSender<Metric>,
        tokio::sync::mpsc::UnboundedReceiver<Metric>,
    > {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        ManagerBuilder::new(intake, sender, receiver)
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
        loop {
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
            }
        }
        Ok(())
    }
}
