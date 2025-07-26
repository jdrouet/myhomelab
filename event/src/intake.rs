use myhomelab_prelude::Healthcheck;

use crate::EventSource;

pub trait IntakeInput {
    type Attrs: std::fmt::Debug + serde::Serialize;

    fn source(&self) -> &EventSource;
    fn message(&self) -> &str;
    fn attributes(&self) -> Option<&Self::Attrs>;
}

pub trait Intake: Healthcheck {
    fn ingest<I: IntakeInput>(&self, input: &I) -> impl Future<Output = anyhow::Result<()>>;
}
