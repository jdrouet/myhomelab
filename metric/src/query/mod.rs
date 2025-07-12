use std::{
    collections::{HashMap, HashSet},
    time::{Duration, SystemTime},
};

use myhomelab_prelude::Healthcheck;

use crate::entity::MetricHeader;

pub trait QueryExecutor: Healthcheck {
    fn execute(
        &self,
        requests: HashMap<Box<str>, Request>,
        timerange: TimeRange,
    ) -> impl Future<Output = anyhow::Result<HashMap<Box<str>, Response>>> + Send;
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct TimeRange {
    pub start: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

impl TimeRange {
    pub fn last_1day() -> Self {
        let start = SystemTime::now() - Duration::from_secs(60 * 60 * 24);
        let start = start
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self { start, end: None }
    }
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
