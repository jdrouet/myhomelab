use std::collections::HashMap;

use myhomelab_metric::entity::{
    Metric, MetricHeader,
    value::{CounterValue, GaugeValue, MetricValue},
};

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Payload {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub counters: HashMap<MetricHeader, MetricValues<u64>>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub gauges: HashMap<MetricHeader, MetricValues<f64>>,
}

impl Payload {
    pub fn from_metrics(metrics: Vec<Metric>) -> Self {
        let mut res = Self::default();
        metrics.into_iter().for_each(|item| match item.value {
            MetricValue::Counter(inner) => {
                let values = res.counters.entry(item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner.0);
            }
            MetricValue::Gauge(inner) => {
                let values = res.gauges.entry(item.header).or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner.0);
            }
        });
        res
    }
}

impl Payload {
    pub fn into_metrics(self) -> impl Iterator<Item = Metric> {
        self.counters
            .into_iter()
            .flat_map(|(header, values)| {
                values.map(move |(timestamp, value)| Metric {
                    header: header.clone(),
                    timestamp,
                    value: MetricValue::Counter(CounterValue(value)),
                })
            })
            .chain(self.gauges.into_iter().flat_map(|(header, values)| {
                values.map(move |(timestamp, value)| Metric {
                    header: header.clone(),
                    timestamp,
                    value: MetricValue::Gauge(GaugeValue(value)),
                })
            }))
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Value<V> {
    pub timestamp: i64,
    pub value: V,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MetricValues<V> {
    pub timestamps: Vec<i64>,
    pub values: Vec<V>,
}

impl<V> Iterator for MetricValues<V> {
    type Item = (i64, V);

    fn next(&mut self) -> Option<Self::Item> {
        let ts = self.timestamps.pop()?;
        let value = self.values.pop()?;

        Some((ts, value))
    }
}
