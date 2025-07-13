use std::collections::{HashMap, HashSet};

use myhomelab_prelude::Healthcheck;
use myhomelab_prelude::time::TimeRange;

use crate::entity::MetricHeader;

pub trait QueryExecutor: Healthcheck {
    fn execute(
        &self,
        requests: HashMap<Box<str>, Request>,
        timerange: TimeRange,
    ) -> impl Future<Output = anyhow::Result<HashMap<Box<str>, Response>>> + Send;
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Request {
    pub kind: RequestKind,
    pub query: Query,
}

impl Request {
    pub fn scalar(query: Query) -> Self {
        Self {
            kind: RequestKind::Scalar,
            query,
        }
    }

    pub fn timeseries(period: u32, query: Query) -> Self {
        Self {
            kind: RequestKind::Timeseries { period },
            query,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum RequestKind {
    Scalar,
    Timeseries { period: u32 },
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Aggregator {
    #[default]
    #[serde(alias = "avg")]
    Average,
    Max,
    Min,
    Sum,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Query {
    #[serde(flatten)]
    pub header: MetricHeader,
    pub aggregator: Aggregator,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Response {
    Scalar(Vec<ScalarResponse>),
    Timeseries(Vec<TimeseriesResponse>),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ScalarResponse {
    pub header: MetricHeader,
    pub value: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TimeseriesResponse {
    pub header: MetricHeader,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<(i64, f64)>,
}
