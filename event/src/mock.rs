#![cfg(feature = "mocks")]

use std::borrow::Cow;

pub struct MockIntakeInput<V: serde::Serialize> {
    pub source: &'static crate::EventSource,
    pub timestamp: u64,
    pub level: crate::EventLevel,
    pub message: Cow<'static, str>,
    pub attributes: Option<V>,
}

impl<V> crate::intake::IntakeInput for MockIntakeInput<V>
where
    V: std::fmt::Debug,
    V: serde::Serialize,
    V: Send + Sync,
{
    type Attrs = V;

    fn source(&self) -> &'static crate::EventSource {
        self.source
    }

    fn level(&self) -> crate::EventLevel {
        self.level
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn attributes(&self) -> Option<&Self::Attrs> {
        self.attributes.as_ref()
    }
}

mockall::mock! {
    pub Event {}

    impl myhomelab_prelude::Healthcheck for Event {
        async fn healthcheck(&self) -> anyhow::Result<()>;
    }

    impl crate::intake::Intake for Event {
        async fn ingest<I>(&self, input: I) -> anyhow::Result<()>
        where
            I: crate::intake::IntakeInput,
            I: 'static;
    }
}
