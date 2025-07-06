use reqwest::StatusCode;

impl myhomelab_dashboard::repository::DashboardRepository for crate::AdapterHttpClient {
    async fn list_dashboards(&self) -> anyhow::Result<Vec<myhomelab_dashboard::entity::Dashboard>> {
        let url = format!("{}/api/dashboards", self.0.base_url);
        let res = self.0.client.get(url).send().await?;
        res.error_for_status_ref()?;
        Ok(res.json().await?)
    }

    async fn find_dashboard_by_id(
        &self,
        uuid: uuid::Uuid,
    ) -> anyhow::Result<Option<myhomelab_dashboard::entity::Dashboard>> {
        let url = format!("{}/api/dashboards/{uuid}", self.0.base_url);
        let res = self.0.client.get(url).send().await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        res.error_for_status_ref()?;
        Ok(res.json().await?)
    }
}
