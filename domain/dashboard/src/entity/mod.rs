use myhomelab_metric::query::{Query, RequestKind};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<DashboardCell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Size {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DashboardCell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub height: Size,
    pub width: Size,
    #[serde(alias = "type")]
    pub kind: RequestKind,
    pub query: Query,
}
