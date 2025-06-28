use std::sync::Arc;

mod metric;

#[derive(Debug)]
pub struct AdapterHttpClientConfig {
    pub base_url: String,
}

impl AdapterHttpClientConfig {
    pub fn build(&self) -> anyhow::Result<AdapterHttpClient> {
        Ok(AdapterHttpClient(Arc::new(Inner {
            base_url: Box::from(self.base_url.as_str()),
            client: reqwest::Client::new(),
        })))
    }
}

#[derive(Debug)]
struct Inner {
    base_url: Box<str>,
    client: reqwest::Client,
}

#[derive(Clone, Debug)]
pub struct AdapterHttpClient(Arc<Inner>);

impl myhomelab_prelude::Healthcheck for AdapterHttpClient {
    async fn healthcheck(&self) -> anyhow::Result<()> {
        let url = format!("{}/api", self.0.base_url);
        let res = self.0.client.head(url).send().await?;
        res.error_for_status()?;
        Ok(())
    }
}
