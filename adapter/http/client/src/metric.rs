use std::collections::HashMap;

use anyhow::Context;
use myhomelab_metric::entity::Metric;
use myhomelab_metric::entity::value::MetricValue;

macro_rules! unwrap_status_error {
    ($res:expr) => {
        if let Err(err) = $res.error_for_status_ref() {
            if let Ok(context) = $res.text().await {
                return Err(anyhow::Error::new(err).context(context));
            } else {
                return Err(anyhow::Error::new(err).context("unable to read response error"));
            }
        }
    };
}

impl myhomelab_metric::intake::Intake for super::AdapterHttpClient {
    async fn ingest<'h>(&self, values: &[Metric<'h, MetricValue>]) -> anyhow::Result<()> {
        use myhomelab_adapter_http_shared::metric::create::Payload;

        let payload = Payload::from_metrics(values.iter());
        let res = self
            .0
            .client
            .post(format!("{}/api/metrics/intake", self.0.base_url))
            .json(&payload)
            .send()
            .await
            .context("sending metrics")?;
        unwrap_status_error!(res);
        Ok(())
    }
}

impl myhomelab_metric::query::QueryExecutor for super::AdapterHttpClient {
    async fn execute(
        &self,
        requests: HashMap<Box<str>, myhomelab_metric::query::Request>,
        range: myhomelab_prelude::time::TimeRange,
    ) -> anyhow::Result<HashMap<Box<str>, myhomelab_metric::query::Response>> {
        use myhomelab_adapter_http_shared::metric::query::BatchQueryParams;

        let query = BatchQueryParams { range, requests };

        let res = self
            .0
            .client
            .post(format!("{}/api/metrics/query", self.0.base_url))
            .json(&query)
            .send()
            .await?;

        unwrap_status_error!(res);
        res.json().await.map_err(anyhow::Error::from)
    }
}
