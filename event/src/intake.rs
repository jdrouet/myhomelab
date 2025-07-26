use myhomelab_prelude::Healthcheck;

use crate::{EventLevel, EventSource};

pub trait IntakeInput: Send + Sync {
    type Attrs: std::fmt::Debug + serde::Serialize;

    fn source(&self) -> &'static EventSource;
    fn timestamp(&self) -> u64;
    fn level(&self) -> EventLevel;
    fn message(&self) -> &str;
    fn attributes(&self) -> Option<&Self::Attrs>;
}

pub trait Intake: Healthcheck {
    fn ingest<I>(&self, input: I) -> impl Future<Output = anyhow::Result<()>> + Send
    where
        I: IntakeInput,
        I: 'static;
}
