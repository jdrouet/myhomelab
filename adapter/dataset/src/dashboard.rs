use myhomelab_dashboard::entity::Dashboard;

impl myhomelab_dashboard::repository::DashboardRepository for crate::AdapterDataset {
    async fn list_dashboards(&self) -> anyhow::Result<Vec<myhomelab_dashboard::entity::Dashboard>> {
        Ok(self.0.dashboards.clone())
    }

    async fn find_dashboard_by_id(&self, uuid: uuid::Uuid) -> anyhow::Result<Option<Dashboard>> {
        Ok(self
            .0
            .dashboards
            .iter()
            .find(|item| item.id == uuid)
            .cloned())
    }
}
