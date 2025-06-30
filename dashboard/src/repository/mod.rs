use crate::entity::Dashboard;

pub trait DashboardRepository {
    fn find_dashboard_by_id(
        &self,
        uuid: uuid::Uuid,
    ) -> impl Future<Output = anyhow::Result<Option<Dashboard>>> + Send;
    fn list_dashboards(&self) -> impl Future<Output = anyhow::Result<Vec<Dashboard>>> + Send;
}

#[cfg(feature = "mocks")]
mockall::mock! {
    pub DashboardRepo {}

    impl DashboardRepository for DashboardRepo {
        fn find_dashboard_by_id(
            &self,
            uuid: uuid::Uuid,
        ) -> impl Future<Output = anyhow::Result<Option<Dashboard>>> + Send;
        fn list_dashboards(&self) -> impl Future<Output = anyhow::Result<Vec<Dashboard>>> + Send;
    }
}
