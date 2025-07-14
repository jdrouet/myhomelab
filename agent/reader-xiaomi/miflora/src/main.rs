use std::time::Duration;

use btleplug::api::{Central, Manager, ScanFilter};
use myhomelab_agent_reader_xiaomi_miflora::device::MiFloraDevice;
// use myhomelab_agent_prelude::mpsc::Sender;

// #[derive(Clone, Debug)]
// struct ConsoleSender;

// impl Sender for ConsoleSender {
//     async fn push(&self, item: myhomelab_metric::entity::Metric) -> anyhow::Result<()> {
//         println!("metric: {item}");
//         Ok(())
//     }
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = btleplug::platform::Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.start_scan(ScanFilter::default()).await?;

    println!("waiting for peripheral");
    tokio::time::sleep(Duration::new(10, 0)).await;

    let peripherals = adapter.peripherals().await?;
    println!("found {} devices", peripherals.len());
    for peripheral in peripherals {
        if let Ok(device) = MiFloraDevice::new(peripheral).await {
            if device.name() != Some("Flower care") {
                continue;
            }
            println!("Name = {:?}", device.name());
            device.connect().await?;
            let battery = device.read_battery().await?;
            println!("Battery = {battery}%");
            let realtime = device.read_realtime_data().await?;
            println!("Realtime = {realtime:?}");
            device.blink().await?;
        }
    }

    Ok(())
}
