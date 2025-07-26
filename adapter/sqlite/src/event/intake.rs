use myhomelab_event::intake::{Intake, IntakeInput};
use sqlx::types::Json;

use crate::Sqlite;

impl Intake for Sqlite {
    async fn ingest<I: IntakeInput>(&self, input: &I) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO events (source, timestamp, level, message, attributes) VALUES (?, ?, ?, ?, ?)")
            .bind(Json(input.source()))
            .bind(input.timestamp() as i64)
            .bind(input.level().as_str())
            .bind(input.message())
            .bind(Json(input.attributes()))
            .execute(self.as_ref())
            .await?;
        Ok(())
    }
}
