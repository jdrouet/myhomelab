#![cfg(feature = "mocks")]

use std::borrow::Cow;

pub struct MockIntakeInput<V: serde::Serialize> {
    pub source: crate::EventSource,
    pub message: Cow<'static, str>,
    pub attributes: Option<V>,
}

impl<V: serde::Serialize> crate::intake::IntakeInput for MockIntakeInput<V> {
    type Attrs = V;

    fn source(&self) -> &crate::EventSource {
        &self.source
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
        async fn ingest<I>(&self, input: I) -> anyhow::Result<()>;
    }
}
