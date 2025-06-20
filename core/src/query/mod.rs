use std::collections::HashMap;

use crate::metric::MetricHeader;

pub trait QueryExecutor {
    fn ingest(
        &self,
        requests: &[Request],
        timerange: (i64, Option<i64>),
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Request {
    kind: RequestKind,
    queries: HashMap<Box<str>, Query>,
}

impl Request {
    pub fn scalar() -> Self {
        Self {
            kind: RequestKind::Scalar,
            queries: HashMap::default(),
        }
    }

    pub fn timeseries() -> Self {
        Self {
            kind: RequestKind::Timeseries,
            queries: HashMap::default(),
        }
    }

    pub fn with_query(mut self, name: impl Into<Box<str>>, query: Query) -> Self {
        self.queries.insert(name.into(), query);
        self
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum RequestKind {
    Scalar,
    Timeseries,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub enum Aggregator {
    #[default]
    Average,
    Max,
    Min,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Query {
    header: MetricHeader,
    aggregator: Aggregator,
}

impl Query {
    pub fn new(header: MetricHeader, aggregator: Aggregator) -> Self {
        Self { header, aggregator }
    }

    pub fn avg(header: MetricHeader) -> Self {
        Self {
            header,
            aggregator: Aggregator::Average,
        }
    }

    pub fn max(header: MetricHeader) -> Self {
        Self {
            header,
            aggregator: Aggregator::Max,
        }
    }

    pub fn min(header: MetricHeader) -> Self {
        Self {
            header,
            aggregator: Aggregator::Min,
        }
    }
}
