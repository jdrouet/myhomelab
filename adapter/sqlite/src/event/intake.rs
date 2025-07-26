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

#[cfg(test)]
mod tests {
    use myhomelab_event::EventSource;
    use myhomelab_event::intake::Intake;

    struct FakeEvent {
        source: myhomelab_event::EventSource,
        level: myhomelab_event::EventLevel,
        message: &'static str,
        timestamp: u64,
        attributes: Option<serde_json::Value>,
    }

    impl myhomelab_event::intake::IntakeInput for FakeEvent {
        type Attrs = serde_json::Value;

        fn source(&self) -> &myhomelab_event::EventSource {
            &self.source
        }
        fn level(&self) -> myhomelab_event::EventLevel {
            self.level
        }

        fn message(&self) -> &str {
            self.message
        }

        fn timestamp(&self) -> u64 {
            self.timestamp
        }

        fn attributes(&self) -> Option<&Self::Attrs> {
            self.attributes.as_ref()
        }
    }

    #[tokio::test]
    async fn should_insert_events() {
        let sqlite = crate::SqliteConfig::default().build().await.unwrap();
        sqlite.prepare().await.unwrap();

        sqlite
            .ingest(&FakeEvent {
                source: EventSource::sensor("fake"),
                level: myhomelab_event::EventLevel::Info,
                message: "Hello World",
                timestamp: 42,
                attributes: Some(serde_json::json!({
                    "foo": "bar",
                })),
            })
            .await
            .unwrap();

        sqlite
            .ingest(&FakeEvent {
                source: EventSource::sensor("fake"),
                level: myhomelab_event::EventLevel::Debug,
                message: "Hello Debug World",
                timestamp: 52,
                attributes: Some(serde_json::json!({
                    "foo": "bar",
                    "baz": 42,
                })),
            })
            .await
            .unwrap();
    }
}
