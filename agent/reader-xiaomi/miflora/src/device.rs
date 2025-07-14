use std::time::Duration;

use anyhow::Context;
use btleplug::api::Characteristic;
use btleplug::api::Peripheral;
use btleplug::api::WriteType;
use uuid::Uuid;

const SERVICE_ID: uuid::Uuid = uuid::uuid!("00001204-0000-1000-8000-00805f9b34fb");
const READ_DATA_UUID: uuid::Uuid = uuid::uuid!("00001a01-0000-1000-8000-00805f9b34fb");
const READ_CMD_UUID: uuid::Uuid = uuid::uuid!("00001a00-0000-1000-8000-00805f9b34fb");
const BATTERY_UUID: uuid::Uuid = uuid::uuid!("00001a02-0000-1000-8000-00805f9b34fb");
// const HISTORY_CONTROL_UUID: uuid::Uuid = uuid::uuid!("00001a10-0000-1000-8000-00805f9b34fb");
// const HISTORY_DATA_UUID: uuid::Uuid = uuid::uuid!("00001a11-0000-1000-8000-00805f9b34fb");
// const DEVICE_TIME_UUID: uuid::Uuid = uuid::uuid!("00001a12-0000-1000-8000-00805f9b34fb");

// const HISTORY_SERVICE_ID: Uuid = uuid::uuid!("00001206-0000-1000-8000-00805f9b34fb");
// service=00001206-0000-1000-8000-00805f9b34fb uuid=00001a10-0000-1000-8000-00805f9b34fb
// service=00001206-0000-1000-8000-00805f9b34fb uuid=00001a11-0000-1000-8000-00805f9b34fb
// service=00001206-0000-1000-8000-00805f9b34fb uuid=00001a12-0000-1000-8000-00805f9b34fb
const HISTORY_CHAR_CTRL: Uuid = uuid::uuid!("00001a10-0000-1000-8000-00805f9b34fb");
const HISTORY_CHAR_DATA: Uuid = uuid::uuid!("00001a11-0000-1000-8000-00805f9b34fb");
// const HISTORY_CHAR_TIME_ID: u16 = 64;

const HISTORY_CMD_READ_INIT: [u8; 3] = [0xA0, 0x00, 0x00];
// const HISTORY_CMD_READ_SUCCESS: [u8; 3] = [0xa2, 0x00, 0x00];

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

enum Mode {
    History,
    Realtime,
}

impl Mode {
    const fn cmd(&self) -> [u8; 2] {
        match self {
            Self::History => [0xc0, 0x1f],
            Self::Realtime => [0xa0, 0x1f],
        }
    }
}

#[derive(Debug)]
pub struct HistoryData {
    pub timestamp: u32,
    pub temperature: f64,
    pub light: u32,
    pub moisture: u8,
    pub conductivity: u16,
}

impl HistoryData {
    fn from_data(data: &[u8]) -> Option<Self> {
        let timestamp =
            u32::from_le_bytes([*data.get(0)?, *data.get(1)?, *data.get(2)?, *data.get(3)?]);
        let temperature = u16::from_le_bytes([*data.get(4)?, *data.get(5)?]) as f64 / 10.0;
        let light = u32::from_le_bytes([*data.get(6)?, *data.get(7)?, *data.get(8)?, 0x00]);
        let moisture = *data.get(9)?;
        let conductivity = u16::from_le_bytes([*data.get(10)?, *data.get(11)?]);

        Some(Self {
            timestamp,
            temperature,
            moisture,
            light,
            conductivity,
        })
    }
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

    fn characteristic(&self, char_id: Uuid) -> anyhow::Result<Characteristic> {
        let list = self.peripheral.characteristics();
        list.into_iter()
            .find(|c| c.uuid == char_id)
            .ok_or_else(|| anyhow::anyhow!("unable to find characteristic {char_id:?}"))
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
        let blink_cmd_char = self.characteristic(READ_CMD_UUID)?;
        self.peripheral
            .write(&blink_cmd_char, &[0xA0, 0xFF], WriteType::WithResponse)
            .await
            .context("couldn't enable blink mode")?;
        Ok(())
    }

    pub async fn read_battery(&self) -> anyhow::Result<u8> {
        let battery_char = self.characteristic(BATTERY_UUID)?;
        let battery_data = self.peripheral.read(&battery_char).await?;
        battery_data
            .get(0)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("invalid response payload"))
    }

    pub async fn read_realtime_data(&self) -> anyhow::Result<RealtimeData> {
        self.set_mode(Mode::Realtime).await?;

        let data_char = self.characteristic(READ_DATA_UUID)?;
        let data = self.peripheral.read(&data_char).await?;

        RealtimeData::from_data(&data).ok_or_else(|| anyhow::anyhow!("unable to read payload"))
    }

    async fn set_mode(&self, mode: Mode) -> anyhow::Result<()> {
        let char = self.characteristic(READ_CMD_UUID)?;
        self.peripheral
            .write(&char, &mode.cmd(), WriteType::WithResponse)
            .await
            .context("changing mode")?;
        let data = self
            .peripheral
            .read(&char)
            .await
            .context("reading current mode")?;
        if !data.eq(&mode.cmd()) {
            return Err(anyhow::anyhow!("invalid mode returned"));
        }
        Ok(())
    }

    pub async fn read_history_data(&self) -> anyhow::Result<Vec<HistoryData>> {
        self.set_mode(Mode::History).await?;

        let ctrl_char = self.characteristic(HISTORY_CHAR_CTRL)?;
        let data_char = self.characteristic(HISTORY_CHAR_DATA)?;

        self.peripheral
            .write(&ctrl_char, &HISTORY_CMD_READ_INIT, WriteType::WithResponse)
            .await
            .context("couldn't initiate history reading")?;

        // request number of history entries
        let count_data = self
            .peripheral
            .read(&data_char)
            .await
            .context("couldn't read history length")?;
        let entry_count = count_data.get(0).copied().unwrap_or(0);
        println!("expecting {entry_count} values");

        let mut entries = Vec::with_capacity(entry_count as usize);

        for idx in 0..entry_count {
            self.peripheral
                .write(&ctrl_char, &[0xA1, idx], WriteType::WithResponse)
                .await
                .context("unable to select new historical value")?;
            tokio::time::sleep(Duration::from_millis(150)).await;
            let data = self
                .peripheral
                .read(&data_char)
                .await
                .context("couldn't read historical value")?;
            if let Some(decoded) = HistoryData::from_data(&data) {
                println!("decoded: {decoded:?}");
                entries.push(decoded);
            } else {
                tracing::warn!(message = "unable to decode history entry", data = ?data);
            }
        }
        Ok(entries)
    }
}
