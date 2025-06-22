use axum::{Json, extract::State, http::StatusCode};
use myhomelab_metric::{
    entity::{
        Metric, MetricHeader,
        value::{CounterValue, GaugeValue, MetricValue},
    },
    intake::Intake,
};

use crate::ServerState;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    pub series: Vec<Serie>,
}

impl Payload {
    fn into_metrics(self) -> Vec<Metric> {
        let mut buf = Vec::new();
        self.series
            .into_iter()
            .for_each(|serie| serie.collect_metrics(&mut buf));
        buf
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Value<V> {
    pub timestamp: i64,
    pub value: V,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MetricValues {
    Counter { values: Vec<Value<u64>> },
    Gauge { values: Vec<Value<f64>> },
}

impl MetricValues {
    fn collect_metrics(self, header: &MetricHeader, buf: &mut Vec<Metric>) {
        match self {
            Self::Counter { values } => {
                values
                    .into_iter()
                    .map(|Value { timestamp, value }| Metric {
                        header: header.clone(),
                        timestamp,
                        value: MetricValue::Counter(CounterValue(value)),
                    })
                    .for_each(|metric| buf.push(metric));
            }
            Self::Gauge { values } => values
                .into_iter()
                .map(|Value { timestamp, value }| Metric {
                    header: header.clone(),
                    timestamp,
                    value: MetricValue::Gauge(GaugeValue(value)),
                })
                .for_each(|metric| buf.push(metric)),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Serie {
    #[serde(flatten)]
    pub header: MetricHeader,
    #[serde(flatten)]
    pub values: MetricValues,
}

impl Serie {
    fn collect_metrics(self, buf: &mut Vec<Metric>) {
        self.values.collect_metrics(&self.header, buf);
    }
}

pub(super) async fn handle<S: ServerState>(
    State(state): State<S>,
    Json(payload): Json<Payload>,
) -> StatusCode {
    let metrics = payload.into_metrics();
    match state.metric_intake().ingest(&metrics).await {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            tracing::error!(message = "unable to ingest metrics", cause = ?err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
