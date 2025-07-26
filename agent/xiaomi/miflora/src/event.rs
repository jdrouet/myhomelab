use std::borrow::Cow;

use btleplug::api::BDAddr;
use myhomelab_event::EventLevel;
use myhomelab_prelude::time::current_timestamp;

const EVENT_SOURCE: myhomelab_event::EventSource = myhomelab_event::EventSource::Sensor {
    name: Cow::Borrowed(crate::DEVICE),
};

fn serialize_address<S>(address: &BDAddr, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let value = address.to_string();
    serializer.serialize_str(value.as_str())
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DeviceAttributes {
    #[serde(serialize_with = "serialize_address")]
    address: BDAddr,
}

#[derive(Debug)]
pub(crate) struct DeviceEvent {
    attrs: DeviceAttributes,
    level: EventLevel,
    message: &'static str,
    timestamp: u64,
}

impl DeviceEvent {
    pub fn new(address: BDAddr, level: EventLevel, message: &'static str) -> Self {
        Self {
            attrs: DeviceAttributes { address },
            level,
            message,
            timestamp: current_timestamp(),
        }
    }
}

impl myhomelab_event::intake::IntakeInput for DeviceEvent {
    type Attrs = DeviceAttributes;

    fn source(&self) -> &'static myhomelab_event::EventSource {
        &EVENT_SOURCE
    }

    fn level(&self) -> myhomelab_event::EventLevel {
        self.level
    }

    fn message(&self) -> &str {
        self.message
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn attributes(&self) -> Option<&Self::Attrs> {
        Some(&self.attrs)
    }
}
