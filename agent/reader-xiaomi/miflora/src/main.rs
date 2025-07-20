use std::time::Duration;

use myhomelab_agent_prelude::collector::TracingCollector;
use myhomelab_agent_prelude::reader::{BuildContext, Reader, ReaderBuilder};
use myhomelab_agent_reader_xiaomi_miflora::MifloraReaderConfig;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = MifloraReaderConfig::default();
    let cancel = CancellationToken::new();
    let build_ctx = BuildContext {
        cancel: cancel.child_token(),
        collector: TracingCollector,
    };
    let sensor = config.build(&build_ctx).await?;
    tokio::time::sleep(Duration::new(180, 0)).await;
    cancel.cancel();
    sensor.wait().await
}
