use std::collections::HashMap;

use myhomelab_metric::entity::value::{CounterValue, GaugeValue, MetricValue};
use myhomelab_metric::entity::{Metric, MetricHeader};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Metrics<V> {
    pub header: MetricHeader,
    pub values: MetricValues<V>,
}

impl Metrics<CounterValue> {
    fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.values.map(move |(timestamp, value)| Metric {
            header: self.header.clone(),
            timestamp,
            value: value.into(),
        })
    }
}

impl Metrics<GaugeValue> {
    fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.values.map(move |(timestamp, value)| Metric {
            header: self.header.clone(),
            timestamp,
            value: value.into(),
        })
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Payload {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counters: Vec<Metrics<CounterValue>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gauges: Vec<Metrics<GaugeValue>>,
}

impl Payload {
    pub fn from_metrics<'a>(metrics: impl Iterator<Item = &'a Metric>) -> Self {
        let mut counters: HashMap<&'a MetricHeader, MetricValues<CounterValue>> =
            Default::default();
        let mut gauges: HashMap<&'a MetricHeader, MetricValues<GaugeValue>> = Default::default();
        metrics.into_iter().for_each(|item| match item.value {
            MetricValue::Counter(inner) => {
                let values = counters.entry(&item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner);
            }
            MetricValue::Gauge(inner) => {
                let values = gauges.entry(&item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner);
            }
        });
        Self {
            counters: Vec::from_iter(counters.into_iter().map(|(header, values)| Metrics {
                header: header.clone(),
                values,
            })),
            gauges: Vec::from_iter(gauges.into_iter().map(|(header, values)| Metrics {
                header: header.clone(),
                values,
            })),
        }
    }
}

impl Payload {
    pub fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.counters
            .into_iter()
            .flat_map(|item| item.into_metrics())
            .chain(self.gauges.into_iter().flat_map(|item| item.into_metrics()))
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MetricValues<V> {
    pub timestamps: Vec<u64>,
    pub values: Vec<V>,
}

impl<V: Into<MetricValue>> Default for MetricValues<V> {
    fn default() -> Self {
        Self {
            timestamps: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<V> Iterator for MetricValues<V> {
    type Item = (u64, V);

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.timestamps.pop()?;
        let value = self.values.pop()?;

        Some((ts, value))
    }
}
