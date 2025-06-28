use myhomelab_metric::query::{Request, TimeRange};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryParams {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requests: Vec<Request>,
    pub range: TimeRange,
}
