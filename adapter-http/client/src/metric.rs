impl myhomelab_metric::intake::Intake for super::AdapterHttpClient {
    async fn ingest(&self, values: Vec<myhomelab_metric::entity::Metric>) -> anyhow::Result<()> {
        use myhomelab_adapter_http_shared::metric::create::Payload;

        let payload = Payload::from_metrics(values);
        let res = self
            .0
            .client
            .post("/api/metrics")
            .json(&payload)
            .send()
            .await?;
        res.error_for_status()?;
        Ok(())
    }
}
