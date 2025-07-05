const TEMPERATURE_INDEX: usize = 6;
const HUMIDITY_INDEX: usize = 8;
const BATTERY_INDEX: usize = 9;

pub(crate) fn read_temperature(data: &[u8]) -> Option<f32> {
    read_f32(data, TEMPERATURE_INDEX)
}

pub(crate) fn read_humidity(data: &[u8]) -> Option<u8> {
    read_u8(data, HUMIDITY_INDEX)
}

pub(crate) fn read_battery(data: &[u8]) -> Option<u8> {
    read_u8(data, BATTERY_INDEX)
}

fn read_u8(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}

fn read_f32(data: &[u8], index: usize) -> Option<f32> {
    let value = [*data.get(index)?, *data.get(index + 1)?];
    Some(i16::from_be_bytes(value) as f32 / 10.0)
}
