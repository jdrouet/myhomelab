use std::borrow::Cow;

pub mod intake;
pub mod mock;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EventSource {
    Sensor { name: Cow<'static, str> },
}

impl EventSource {
    #[inline]
    pub fn sensor(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Sensor { name: name.into() }
    }
}
