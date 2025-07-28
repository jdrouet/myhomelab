use std::borrow::Cow;
use std::sync::Arc;

use myhomelab_prelude::time::current_timestamp;

use crate::Device;

const EVENT_SOURCE: myhomelab_event::EventSource = myhomelab_event::EventSource::Sensor {
    name: Cow::Borrowed(crate::DEVICE),
};

#[derive(Debug)]
pub(crate) struct DeviceDiscoveredEvent {
    timestamp: u64,
    device: Arc<Device>,
}

impl DeviceDiscoveredEvent {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            timestamp: current_timestamp(),
            device,
        }
    }
}

impl myhomelab_event::intake::IntakeInput for DeviceDiscoveredEvent {
    type Attrs = Device;

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
        Some(self.device.as_ref())
    }
}
