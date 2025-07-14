use btleplug::api::{Central, CentralEvent, Manager, ScanFilter};
use btleplug::platform::{Adapter, Peripheral};
use myhomelab_agent_reader_xiaomi_miflora::device::MiFloraDevice;
use tokio_stream::StreamExt;
// use myhomelab_agent_prelude::mpsc::Sender;

// #[derive(Clone, Debug)]
// struct ConsoleSender;

// impl Sender for ConsoleSender {
//     async fn push(&self, item: myhomelab_metric::entity::Metric) ->
// anyhow::Result<()> {         println!("metric: {item}");
//         Ok(())
//     }
// }

async fn find_miflora(adapter: &Adapter) -> anyhow::Result<Peripheral> {
    use btleplug::api::Peripheral as _;

    let mut events = adapter.events().await?;
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let peripheral = adapter.peripheral(&id).await?;
                let props = peripheral.properties().await?;
                let name = props.and_then(|props| props.local_name);
                if name.as_deref() == Some("Flower care") {
                    return Ok(peripheral);
                }
            }
            _ => {}
        }
    }
    Err(anyhow::anyhow!("device not found"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = btleplug::platform::Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.start_scan(ScanFilter::default()).await?;

    let peripheral = find_miflora(&adapter).await?;
    let device = MiFloraDevice::new(&peripheral).await?;
    println!("device found");
    device.connect().await?;
    let battery = device.read_battery().await?;
    println!("Battery = {battery}%");
    let realtime = device.read_realtime_data().await?;
    println!("Realtime = {realtime:?}");
    // device.blink().await?;
    // let history = device.read_history_data().await?;
    // println!("History = {history:#?}");

    Ok(())
}
