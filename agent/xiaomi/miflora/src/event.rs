use std::borrow::Cow;

use btleplug::api::BDAddr;
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
pub(crate) struct EventAttributes {
    #[serde(serialize_with = "serialize_address")]
    address: BDAddr,
}

#[derive(Debug)]
pub(crate) struct DeviceDiscoveredEvent {
    timestamp: u64,
    attrs: EventAttributes,
}

impl DeviceDiscoveredEvent {
    pub fn new(address: BDAddr) -> Self {
        Self {
            timestamp: current_timestamp(),
            attrs: EventAttributes { address },
        }
    }
}

impl myhomelab_event::intake::IntakeInput for DeviceDiscoveredEvent {
    type Attrs = EventAttributes;

    fn source(&self) -> &'static myhomelab_event::EventSource {
        &EVENT_SOURCE
    }

    fn level(&self) -> myhomelab_event::EventLevel {
        myhomelab_event::EventLevel::Info
    }

    fn message(&self) -> &str {
        "device discovered"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn attributes(&self) -> Option<&Self::Attrs> {
        Some(&self.attrs)
    }
}
