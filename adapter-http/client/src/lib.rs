use std::sync::Arc;

mod metric;

#[derive(Debug)]
pub struct AdapterHttpClientConfig {
    pub base_url: String,
}

impl AdapterHttpClientConfig {
    pub fn build(&self) -> anyhow::Result<AdapterHttpClient> {
        Ok(AdapterHttpClient(Arc::new(Inner {
            client: reqwest::Client::new(),
        })))
    }
}

#[derive(Debug)]
struct Inner {
    client: reqwest::Client,
}

#[derive(Clone, Debug)]
pub struct AdapterHttpClient(Arc<Inner>);

impl myhomelab_prelude::Healthcheck for AdapterHttpClient {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        let res = self.0.client.head("/api").send().await?;
        res.error_for_status()?;
        Ok(())
    }
}
