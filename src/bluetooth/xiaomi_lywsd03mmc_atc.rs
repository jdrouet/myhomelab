use opentelemetry::metrics::Gauge;

const SERVICE_ID: uuid::Uuid = uuid::Uuid::from_u128(488837762788578050050668711589115);

#[derive(Debug)]
pub(crate) struct XiaomiLywsd03mmcAtcCollector {
    temperature: Gauge<f64>,
    humidity: Gauge<f64>,
    battery: Gauge<f64>,
}

impl Default for XiaomiLywsd03mmcAtcCollector {
    fn default() -> Self {
        let meter = opentelemetry::global::meter("xiaomi-lywsd03mmc-atc");

        Self {
            temperature: meter
                .f64_gauge("measurement.temperature")
                .with_unit("degree celcius")
                .build(),
            humidity: meter
                .f64_gauge("measurement.humidity")
                .with_unit("percentage")
                .build(),
            battery: meter
                .f64_gauge("system.battery")
                .with_unit("percentage")
                .build(),
        }
    }
}

impl XiaomiLywsd03mmcAtcCollector {
    pub async fn collect(
        &self,
        device: &bluer::Device,
        attributes: &[opentelemetry::KeyValue],
    ) -> anyhow::Result<bool> {
        let data = device.service_data().await?;
        let Some(data) = data.and_then(|mut data| data.remove(&SERVICE_ID)) else {
            return Ok(false);
        };

        if let Some(value) = read_temperature(&data) {
            self.temperature.record(value, attributes);
        }
        if let Some(value) = read_humidity(&data) {
            self.humidity.record(value, attributes);
        }
        if let Some(value) = read_battery(&data) {
            self.battery.record(value, attributes);
        }

        Ok(true)
    }
}

const TEMPERATURE_INDEX: usize = 6;
const HUMIDITY_INDEX: usize = 8;
const BATTERY_INDEX: usize = 9;

fn read_temperature(data: &[u8]) -> Option<f64> {
    let index = TEMPERATURE_INDEX;
    let value = [*data.get(index)?, *data.get(index + 1)?];
    Some(i16::from_be_bytes(value) as f64 / 10.0)
}

fn read_humidity(data: &[u8]) -> Option<f64> {
    read_u8(data, HUMIDITY_INDEX).map(|v| v as f64)
}

fn read_battery(data: &[u8]) -> Option<f64> {
    read_u8(data, BATTERY_INDEX).map(|v| v as f64)
}

fn read_u8(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}
