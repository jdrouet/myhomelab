use std::collections::HashMap;

use myhomelab_metric::entity::value::MetricValue;
use myhomelab_metric::entity::{Metric, MetricHeader};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Metrics<V> {
    pub header: MetricHeader,
    pub values: MetricValues<V>,
}

impl Metrics<u64> {
    fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.values.map(move |(timestamp, value)| Metric {
            header: self.header.clone(),
            timestamp,
            value: MetricValue::counter(value),
        })
    }
}

impl Metrics<f64> {
    fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.values.map(move |(timestamp, value)| Metric {
            header: self.header.clone(),
            timestamp,
            value: MetricValue::gauge(value),
        })
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Payload {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counters: Vec<Metrics<u64>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gauges: Vec<Metrics<f64>>,
}

impl Payload {
    pub fn from_metrics(metrics: Vec<Metric>) -> Self {
        let mut counters: HashMap<MetricHeader, MetricValues<u64>> = Default::default();
        let mut gauges: HashMap<MetricHeader, MetricValues<f64>> = Default::default();
        metrics.into_iter().for_each(|item| match item.value {
            MetricValue::Counter(inner) => {
                let values = counters.entry(item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner.0);
            }
            MetricValue::Gauge(inner) => {
                let values = gauges.entry(item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner.0);
            }
        });
        Self {
            counters: Vec::from_iter(
                counters
                    .into_iter()
                    .map(|(header, values)| Metrics { header, values }),
            ),
            gauges: Vec::from_iter(
                gauges
                    .into_iter()
                    .map(|(header, values)| Metrics { header, values }),
            ),
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

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MetricValues<V> {
    pub timestamps: Vec<u64>,
    pub values: Vec<V>,
}

impl<V> Iterator for MetricValues<V> {
    type Item = (u64, V);

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.timestamps.pop()?;
        let value = self.values.pop()?;

        Some((ts, value))
    }
}
