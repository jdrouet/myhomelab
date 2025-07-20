#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct CounterValue(pub u64);

impl From<u64> for CounterValue {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct GaugeValue(pub f64);

impl From<f64> for GaugeValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Counter(CounterValue),
    Gauge(GaugeValue),
}

impl From<CounterValue> for MetricValue {
    fn from(value: CounterValue) -> Self {
        Self::Counter(value)
    }
}

impl From<GaugeValue> for MetricValue {
    fn from(value: GaugeValue) -> Self {
        Self::Gauge(value)
    }
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
