pub mod event;
pub mod metric;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct SqliteConfig {
    #[serde(default)]
    pub path: Option<String>,
}

impl SqliteConfig {
    pub async fn build(&self) -> sqlx::Result<Sqlite> {
        match self.path.as_deref() {
            None | Some(":memory:") => sqlx::sqlite::SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .connect(":memory:")
                .await
                .map(Sqlite::from),
            Some(other) => sqlx::sqlite::SqlitePoolOptions::new()
                .connect_with(
                    sqlx::sqlite::SqliteConnectOptions::new()
                        .create_if_missing(true)
                        .filename(other),
                )
                .await
                .map(Sqlite::from),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sqlite {
    pool: sqlx::SqlitePool,
}

impl From<sqlx::SqlitePool> for Sqlite {
    fn from(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }
}

impl AsRef<sqlx::SqlitePool> for Sqlite {
    fn as_ref(&self) -> &sqlx::SqlitePool {
        &self.pool
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
