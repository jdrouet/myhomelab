mod intake;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Clone, Debug, Default)]
pub struct SqliteConfig {
    pub url: Option<String>,
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
