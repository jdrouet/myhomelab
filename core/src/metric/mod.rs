use std::collections::HashMap;

pub mod tag;
pub mod value;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MetricHeader {
    pub name: Box<str>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<Box<str>, tag::TagValue>,
}

impl MetricHeader {
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            tags: Default::default(),
        }
    }

    pub fn with_tag(mut self, name: impl Into<Box<str>>, value: impl Into<tag::TagValue>) -> Self {
        self.tags.insert(name.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Metric {
    #[serde(flatten)]
    pub header: MetricHeader,
    pub timestamp: i64,
    pub value: value::MetricValue,
}

impl Metric {
    pub fn as_counter(&self) -> Option<MetricRef<'_, value::CounterValue>> {
        match self.value {
            value::MetricValue::Counter(ref value) => Some(MetricRef {
                header: &self.header,
                timestamp: self.timestamp,
                value,
            }),
            _ => None,
        }
    }

    pub fn as_gauge(&self) -> Option<MetricRef<'_, value::GaugeValue>> {
        match self.value {
            value::MetricValue::Gauge(ref value) => Some(MetricRef {
                header: &self.header,
                timestamp: self.timestamp,
                value,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MetricRef<'a, V> {
    pub header: &'a MetricHeader,
    pub timestamp: i64,
    pub value: &'a V,
}
