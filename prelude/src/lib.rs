use std::str::FromStr;

use anyhow::Context;

pub trait Healthcheck {
    fn healthcheck(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub trait FromEnv: Sized {
    fn from_env() -> anyhow::Result<Self>;
}

pub fn parse_from_env<V: FromStr>(name: &str) -> anyhow::Result<Option<V>>
where
    V::Err: Into<anyhow::Error>,
    V::Err: std::error::Error,
    V::Err: Send + Sync + 'static,
{
    let Ok(value) = std::env::var(name) else {
        return Ok(None);
    };

    value
        .parse::<V>()
        .with_context(|| format!("unable to parse {name:?} value"))
        .map(Some)
}

pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backward")
        .as_secs()
}
