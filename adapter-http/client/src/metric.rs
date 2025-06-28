use anyhow::Context;
use myhomelab_adapter_http_shared::metric::query::SingleQueryParams;

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
    async fn ingest(&self, values: Vec<myhomelab_metric::entity::Metric>) -> anyhow::Result<()> {
        use myhomelab_adapter_http_shared::metric::create::Payload;

        let payload = Payload::from_metrics(values);
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
        requests: Vec<myhomelab_metric::query::Request>,
        range: myhomelab_metric::query::TimeRange,
    ) -> anyhow::Result<Vec<myhomelab_metric::query::Response>> {
        use myhomelab_adapter_http_shared::metric::query::BatchQueryParams;

        let query = BatchQueryParams { range, requests };

        let res = match SingleQueryParams::try_from(query) {
            Ok(single) => {
                let query = serde_qs::to_string(&single).context("serializing query")?;
                self.0
                    .client
                    .get(format!("{}/api/metrics/query?{query}", self.0.base_url))
                    .send()
                    .await?
            }
            Err(batch) => {
                self.0
                    .client
                    .post(format!("{}/api/metrics/query", self.0.base_url))
                    .json(&batch)
                    .send()
                    .await?
            }
        };

        unwrap_status_error!(res);
        res.json().await.map_err(anyhow::Error::from)
    }
}
