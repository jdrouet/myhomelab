#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    PartialOrd,
    derive_more::From,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct CounterValue(pub u64);

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    PartialOrd,
    derive_more::From,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
pub struct GaugeValue(pub f64);

#[derive(
    Clone, Copy, Debug, PartialEq, derive_more::From, serde::Deserialize, serde::Serialize,
)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Counter(CounterValue),
    Gauge(GaugeValue),
}

impl MetricValue {
    pub fn counter(value: u64) -> Self {
        Self::Counter(CounterValue(value))
    }

    pub fn gauge(value: f64) -> Self {
        Self::Gauge(GaugeValue(value))
    }
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Counter(CounterValue(inner)) => write!(f, "Counter({inner})"),
            Self::Gauge(GaugeValue(inner)) => write!(f, "Gauge({inner})"),
        }
    }
}
