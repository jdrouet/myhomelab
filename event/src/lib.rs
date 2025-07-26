use std::borrow::Cow;

pub mod intake;
pub mod mock;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EventSource {
    Sensor { name: Cow<'static, str> },
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventLevel {
    Info,
    Debug,
    Warning,
    Error,
}

impl EventLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl EventSource {
    #[inline]
    pub fn sensor(name: impl Into<Cow<'static, str>>) -> Self {
        Self::Sensor { name: name.into() }
    }
}
