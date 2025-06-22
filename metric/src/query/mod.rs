use std::collections::{HashMap, HashSet};

use crate::entity::MetricHeader;

pub trait QueryExecutor {
    fn execute(
        &self,
        requests: &[Request],
        timerange: TimeRange,
    ) -> impl Future<Output = anyhow::Result<Vec<Response>>> + Send;
}

#[derive(Clone, Copy, Debug)]
pub struct TimeRange {
    pub start: i64,
    pub end: Option<i64>,
}

impl From<i64> for TimeRange {
    fn from(value: i64) -> Self {
        Self {
            start: value,
            end: None,
        }
    }
}

impl From<(i64, i64)> for TimeRange {
    fn from((start, end): (i64, i64)) -> Self {
        Self {
            start,
            end: Some(end),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Request {
    pub kind: RequestKind,
    pub queries: HashMap<Box<str>, Query>,
}

impl Request {
    pub fn scalar() -> Self {
        Self {
            kind: RequestKind::Scalar,
            queries: HashMap::default(),
        }
    }

    pub fn timeseries(period: u32) -> Self {
        Self {
            kind: RequestKind::Timeseries { period },
            queries: HashMap::default(),
        }
    }

    pub fn with_query(mut self, name: impl Into<Box<str>>, query: Query) -> Self {
        self.queries.insert(name.into(), query);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum RequestKind {
    Scalar,
    Timeseries { period: u32 },
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub enum Aggregator {
    #[default]
    Average,
    Max,
    Min,
    Sum,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Query {
    pub header: MetricHeader,
    pub aggregator: Aggregator,
    pub group_by: HashSet<Box<str>>,
}

impl Query {
    pub fn new(header: MetricHeader, aggregator: Aggregator) -> Self {
        Self {
            header,
            aggregator,
            group_by: Default::default(),
        }
    }

    pub fn avg(header: MetricHeader) -> Self {
        Self::new(header, Aggregator::Average)
    }

    pub fn max(header: MetricHeader) -> Self {
        Self::new(header, Aggregator::Max)
    }

    pub fn min(header: MetricHeader) -> Self {
        Self::new(header, Aggregator::Min)
    }

    pub fn sum(header: MetricHeader) -> Self {
        Self::new(header, Aggregator::Sum)
    }

    pub fn with_group_by<V: Into<Box<str>>>(mut self, fields: impl Iterator<Item = V>) -> Self {
        self.group_by = HashSet::from_iter(fields.map(|item| item.into()));
        self
    }
}

#[derive(Debug)]
pub struct Response {
    pub kind: RequestKind,
    pub queries: HashMap<Box<str>, QueryResponse>,
}

#[derive(Debug)]
pub enum QueryResponse {
    Scalar(Vec<ScalarQueryResponse>),
    Timeseries(Vec<TimeseriesQueryResponse>),
}

#[derive(Debug)]
pub struct ScalarQueryResponse {
    pub header: MetricHeader,
    pub value: f64,
}

#[derive(Debug)]
pub struct TimeseriesQueryResponse {
    pub header: MetricHeader,
    pub values: Vec<(i64, f64)>,
}
