use std::collections::HashMap;

use myhomelab_metric::query::{Request, TimeRange};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BatchQueryParams {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub requests: HashMap<Box<str>, Request>,
    pub range: TimeRange,
}
