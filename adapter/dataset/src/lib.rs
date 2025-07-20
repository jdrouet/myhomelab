use std::sync::Arc;

use myhomelab_dashboard::entity::Dashboard;

mod dashboard;

#[derive(Debug, Default, serde::Deserialize)]
pub struct AdapterDatasetConfig(Arc<Content>);

impl AdapterDatasetConfig {
    pub fn build(&self) -> AdapterDataset {
        AdapterDataset(self.0.clone())
    }
}

#[derive(Clone, Debug, Default)]
pub struct AdapterDataset(Arc<Content>);

#[derive(Debug, Default, serde::Deserialize)]
struct Content {
    #[serde(default)]
    dashboards: Vec<Dashboard>,
}
