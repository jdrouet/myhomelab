use std::collections::HashMap;

use myhomelab_metric::query::{Query, Request, RequestKind, TimeRange};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BatchQueryParams {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requests: Vec<Request>,
    pub range: TimeRange,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SingleQueryParams {
    pub mode: RequestKind,
    #[serde(flatten)]
    pub query: Query,
    pub range: TimeRange,
}

impl From<SingleQueryParams> for BatchQueryParams {
    fn from(value: SingleQueryParams) -> Self {
        let mut queries = HashMap::with_capacity(1);
        queries.insert("single".into(), value.query);
        BatchQueryParams {
            requests: vec![Request {
                kind: value.mode,
                queries,
            }],
            range: value.range,
        }
    }
}

impl TryFrom<BatchQueryParams> for SingleQueryParams {
    type Error = BatchQueryParams;

    fn try_from(mut value: BatchQueryParams) -> Result<Self, Self::Error> {
        if value.requests.len() != 1 || value.requests[0].queries.len() != 1 {
            return Err(value);
        }
        let req = value.requests.pop().unwrap();
        let query = req.queries.into_values().next().unwrap();
        Ok(Self {
            mode: req.kind,
            query,
            range: value.range,
        })
    }
}
