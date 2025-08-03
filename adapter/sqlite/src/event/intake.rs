use myhomelab_event::intake::{Intake, IntakeInput};
use sqlx::types::Json;

use crate::Sqlite;

impl Intake for Sqlite {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn ingest<I>(&self, input: I) -> anyhow::Result<()>
    where
        I: IntakeInput,
    {
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
    use std::borrow::Cow;

    use myhomelab_event::intake::Intake;

    const EVENT_SOURCE: myhomelab_event::EventSource = myhomelab_event::EventSource::Sensor {
        name: Cow::Borrowed("fake"),
    };

    struct FakeEvent {
        level: myhomelab_event::EventLevel,
        message: &'static str,
        timestamp: u64,
        attributes: Option<serde_json::Value>,
    }

    impl myhomelab_event::intake::IntakeInput for FakeEvent {
        type Attrs = serde_json::Value;

        fn source(&self) -> &'static myhomelab_event::EventSource {
            &EVENT_SOURCE
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
            .ingest(FakeEvent {
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
            .ingest(FakeEvent {
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
