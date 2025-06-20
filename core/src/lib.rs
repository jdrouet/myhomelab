use std::collections::HashSet;

pub mod intake;

#[derive(Clone, Debug)]
pub enum TagValueArray {
    Text(Box<str>),
    Integer(i64),
}

#[derive(Clone, Debug)]
pub enum TagValue {
    Text(Box<str>),
    Integer(i64),
    Array(TagValueArray),
}

#[derive(Clone, Debug)]
pub struct Metric {
    pub name: Box<str>,
    pub timestamp: i64,
    pub tags: HashSet<Box<str>, TagValue>,
    pub value: MetricValue,
}

#[derive(Clone, Debug)]
pub enum MetricValue {
    Count(u64),
    Gauge(f64),
}
