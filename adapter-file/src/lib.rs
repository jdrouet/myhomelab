use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use myhomelab_dashboard::entity::Dashboard;

mod dashboard;

#[derive(Debug)]
pub struct AdapterFileConfig {
    pub path: Option<PathBuf>,
}

impl myhomelab_prelude::FromEnv for AdapterFileConfig {
    fn from_env() -> anyhow::Result<Self> {
        let path = myhomelab_prelude::parse_from_env("MYHOMELAB_DATASET_PATH")?;
        Ok(Self { path })
    }
}

impl AdapterFileConfig {
    pub fn build(&self) -> anyhow::Result<AdapterFile> {
        if let Some(ref path) = self.path {
            let content = Content::from_path(path)?;
            Ok(AdapterFile(Arc::new(content)))
        } else {
            Ok(AdapterFile::default())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AdapterFile(Arc<Content>);

#[derive(Debug, Default, serde::Deserialize)]
struct Content {
    #[serde(default)]
    dashboards: Vec<Dashboard>,
}

impl Content {
    fn from_path(path: &Path) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        toml::from_str(&data).context("unable to parse file")
    }
}
