use std::borrow::Cow;
use std::collections::HashMap;

use myhomelab_metric::entity::value::{CounterValue, GaugeValue, MetricValue};
use myhomelab_metric::entity::{Metric, MetricHeader, MetricTags};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Metrics<'h, V> {
    pub name: Cow<'h, str>,
    pub tags: Cow<'h, MetricTags>,
    pub values: MetricValues<V>,
}

impl<'h> Metrics<'h, CounterValue> {
    fn iter<'a>(&'h self) -> impl Iterator<Item = Metric<'a, MetricValue>>
    where
        'h: 'a,
    {
        self.values.iter().map(|(timestamp, value)| Metric {
            name: Cow::Borrowed(&self.name),
            tags: Cow::Borrowed(&self.tags),
            timestamp,
            value,
        })
    }
}

impl<'h> Metrics<'h, GaugeValue> {
    fn iter<'a>(&'h self) -> impl Iterator<Item = Metric<'a, MetricValue>>
    where
        'h: 'a,
    {
        self.values.iter().map(|(timestamp, value)| Metric {
            name: Cow::Borrowed(&self.name),
            tags: Cow::Borrowed(&self.tags),
            timestamp,
            value,
        })
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Payload<'h> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counters: Vec<Metrics<'h, CounterValue>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gauges: Vec<Metrics<'h, GaugeValue>>,
}

impl<'h> Payload<'h> {
    pub fn from_metrics(metrics: impl Iterator<Item = &'h Metric<'h, MetricValue>>) -> Self {
        let mut counters: HashMap<MetricHeader<'h>, MetricValues<CounterValue>> =
            Default::default();
        let mut gauges: HashMap<MetricHeader, MetricValues<GaugeValue>> = Default::default();
        metrics.into_iter().for_each(|item| match item.value {
            MetricValue::Counter(inner) => {
                let values = counters
                    .entry(MetricHeader::new(&item.name, &item.tags))
                    .or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner);
            }
            MetricValue::Gauge(inner) => {
                let values = gauges
                    .entry(MetricHeader::new(&item.name, &item.tags))
                    .or_default();
                values.timestamps.push(item.timestamp);
                values.values.push(inner);
            }
        });
        Self {
            counters: Vec::from_iter(counters.into_iter().map(|(header, values)| Metrics {
                name: Cow::Borrowed(header.name),
                tags: Cow::Borrowed(header.tags),
                values,
            })),
            gauges: Vec::from_iter(gauges.into_iter().map(|(header, values)| Metrics {
                name: Cow::Borrowed(header.name),
                tags: Cow::Borrowed(header.tags),
                values,
            })),
        }
    }
}

impl<'h> Payload<'h> {
    pub fn metrics(&'h self) -> impl Iterator<Item = Metric<'h, MetricValue>> {
        self.counters
            .iter()
            .flat_map(|item| item.iter())
            .chain(self.gauges.iter().flat_map(|item| item.iter()))
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

impl<V: Copy + Into<MetricValue>> MetricValues<V> {
    fn iter(&self) -> impl Iterator<Item = (u64, MetricValue)> {
        self.timestamps
            .iter()
            .copied()
            .zip(self.values.iter().copied().map(Into::into))
    }
}
