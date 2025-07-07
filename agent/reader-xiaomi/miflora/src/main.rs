use myhomelab_agent_prelude::{mpsc::Sender, reader::Reader};
use myhomelab_agent_reader_xiaomi_miflora::ReaderConfig;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
struct ConsoleSender;

impl Sender for ConsoleSender {
    async fn push(&self, item: myhomelab_metric::entity::Metric) -> anyhow::Result<()> {
        println!("metric: {item}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = CancellationToken::new();
    let reader = ReaderConfig::default().build().await?;
    reader.run(token, ConsoleSender).await
}
