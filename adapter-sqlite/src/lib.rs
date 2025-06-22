mod intake;
mod query;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Clone, Debug, Default)]
pub struct SqliteConfig {
    pub url: Option<String>,
}

impl myhomelab_prelude::FromEnv for SqliteConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            url: std::env::var("MYHOMELAB_SQLITE_PATH").ok(),
        })
    }
}

impl SqliteConfig {
    pub async fn build(&self) -> sqlx::Result<Sqlite> {
        match self.url.as_deref() {
            None | Some(":memory:") => sqlx::sqlite::SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .connect(":memory:")
                .await
                .map(Sqlite),
            Some(other) => sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(
                    sqlx::sqlite::SqliteConnectOptions::new()
                        .create_if_missing(true)
                        .filename(other),
                )
                .await
                .map(Sqlite),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sqlite(sqlx::SqlitePool);

impl AsRef<sqlx::SqlitePool> for Sqlite {
    fn as_ref(&self) -> &sqlx::SqlitePool {
        &self.0
    }
}

impl Sqlite {
    pub async fn prepare(&self) -> Result<(), sqlx::migrate::MigrateError> {
        MIGRATOR.run(self.as_ref()).await
    }
}

impl myhomelab_prelude::Healthcheck for Sqlite {
    #[tracing::instrument(skip_all, level = "DEBUG", ret)]
    async fn healthcheck(&self) -> anyhow::Result<()> {
        sqlx::query("select 1").execute(self.as_ref()).await?;
        Ok(())
    }
}
