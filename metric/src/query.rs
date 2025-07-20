use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use myhomelab_prelude::Healthcheck;
use myhomelab_prelude::time::TimeRange;

use crate::entity::MetricTags;

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

    pub fn timeseries(query: Query) -> Self {
        Self {
            kind: RequestKind::Timeseries,
            query,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "name", rename_all = "kebab-case")]
pub enum RequestKind {
    Scalar,
    Timeseries,
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
    pub name: Cow<'static, str>,
    pub tags: MetricTags,
    pub aggregator: Aggregator,
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub group_by: HashSet<Box<str>>,
}

impl Query {
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        tags: MetricTags,
        aggregator: Aggregator,
    ) -> Self {
        Self {
            name: name.into(),
            tags,
            aggregator,
            group_by: Default::default(),
        }
    }

    pub fn avg(name: impl Into<Cow<'static, str>>, tags: MetricTags) -> Self {
        Self::new(name, tags, Aggregator::Average)
    }

    pub fn max(name: impl Into<Cow<'static, str>>, tags: MetricTags) -> Self {
        Self::new(name, tags, Aggregator::Max)
    }

    pub fn min(name: impl Into<Cow<'static, str>>, tags: MetricTags) -> Self {
        Self::new(name, tags, Aggregator::Min)
    }

    pub fn sum(name: impl Into<Cow<'static, str>>, tags: MetricTags) -> Self {
        Self::new(name, tags, Aggregator::Sum)
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
    pub name: Cow<'static, str>,
    pub tags: MetricTags,
    pub value: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TimeseriesResponse {
    pub name: Cow<'static, str>,
    pub tags: MetricTags,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<(u64, f64)>,
}
