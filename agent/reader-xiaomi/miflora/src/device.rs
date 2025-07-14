use btleplug::api::Peripheral;
use btleplug::api::WriteType;

const SERVICE_ID: uuid::Uuid = uuid::uuid!("00001204-0000-1000-8000-00805f9b34fb");
const READ_DATA_UUID: uuid::Uuid = uuid::uuid!("00001a01-0000-1000-8000-00805f9b34fb");
const READ_CMD_UUID: uuid::Uuid = uuid::uuid!("00001a00-0000-1000-8000-00805f9b34fb");
const BATTERY_UUID: uuid::Uuid = uuid::uuid!("00001a02-0000-1000-8000-00805f9b34fb");

#[cfg(target_os = "macos")]
fn read_address<P: Peripheral>(peripheral: &P) -> String {
    let tmp = peripheral.id().to_string();
    tmp.chars()
        .into_iter()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(12)
        .enumerate()
        .fold(String::with_capacity(17), |mut acc, (index, c)| {
            if index > 0 && index % 2 == 0 {
                acc.push(':');
            }
            acc.push(c.to_ascii_uppercase());
            acc
        })
}

#[cfg(not(target_os = "macos"))]
fn read_address<P: Peripheral>(peripheral: P) -> String {
    peripheral.address().to_string()
}

#[derive(Debug)]
pub struct RealtimeData {
    pub temperature: f64,
    pub light: u32,
    pub moisture: u8,
    pub conductivity: u16,
}

impl RealtimeData {
    fn from_data(data: &[u8]) -> Option<Self> {
        let temperature = u16::from_le_bytes([*data.get(0)?, *data.get(1)?]) as f64 / 10.0;
        let light = u32::from_le_bytes([*data.get(3)?, *data.get(4)?, *data.get(5)?, 0x00]); // 24-bit value
        let moisture = *data.get(7)?;
        let conductivity = u16::from_le_bytes([*data.get(8)?, *data.get(9)?]);

        Some(Self {
            temperature,
            light,
            moisture,
            conductivity,
        })
    }
}

#[derive(Debug)]
pub struct MiFloraDevice<P: Peripheral> {
    address: String,
    name: Option<String>,
    peripheral: P,
}

impl<P: Peripheral> MiFloraDevice<P> {
    pub async fn new(peripheral: P) -> anyhow::Result<Self> {
        let address = read_address(&peripheral);
        let name = peripheral
            .properties()
            .await?
            .and_then(|props| props.local_name);
        Ok(Self {
            address,
            name,
            peripheral,
        })
    }

    pub fn address(&self) -> &str {
        self.address.as_str()
    }

    pub async fn check(&self) -> anyhow::Result<()> {
        let services = self.peripheral.services();
        services
            .iter()
            .find(|s| s.uuid == SERVICE_ID)
            .ok_or_else(|| anyhow::anyhow!("service not found"))?;
        Ok(())
    }

    pub async fn connect(&self) -> anyhow::Result<()> {
        self.peripheral.connect().await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> anyhow::Result<()> {
        self.peripheral.disconnect().await?;
        Ok(())
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn into_peripheral(self) -> P {
        self.peripheral
    }

    pub async fn blink(&self) -> anyhow::Result<()> {
        let blink_cmd_char = self
            .peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == READ_CMD_UUID)
            .ok_or_else(|| anyhow::anyhow!("characteristic not found"))?;
        self.peripheral
            .write(&blink_cmd_char, &[0xA0, 0xFF], WriteType::WithResponse)
            .await?;
        Ok(())
    }

    pub async fn read_battery(&self) -> anyhow::Result<u8> {
        let battery_char = self
            .peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == BATTERY_UUID)
            .ok_or_else(|| anyhow::anyhow!("characteristic not found"))?;
        let battery_data = self.peripheral.read(&battery_char).await?;
        battery_data
            .get(0)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("invalid response payload"))
    }

    pub async fn read_realtime_data(&self) -> anyhow::Result<RealtimeData> {
        let read_cmd_char = self
            .peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == READ_CMD_UUID)
            .ok_or_else(|| anyhow::anyhow!("characteristic not found"))?;
        self.peripheral
            .write(&read_cmd_char, &[0xA0, 0x1F], WriteType::WithResponse)
            .await?;

        let data_char = self
            .peripheral
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == READ_DATA_UUID)
            .ok_or_else(|| anyhow::anyhow!("characteristic not found"))?;
        let data = self.peripheral.read(&data_char).await?;

        RealtimeData::from_data(&data).ok_or_else(|| anyhow::anyhow!("unable to read payload"))
    }
}
