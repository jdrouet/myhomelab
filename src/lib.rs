mod otel;

pub trait Configurable: Sized {
    fn from_env() -> anyhow::Result<Self>;
}

pub struct ApplicationConfig {
    otel: crate::otel::OtelConfig,
}

impl Configurable for ApplicationConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            otel: crate::otel::OtelConfig::from_env()?,
        })
    }
}

impl ApplicationConfig {
    pub fn build(&self) -> anyhow::Result<Application> {
        self.otel.install()?;

        Ok(Application {})
    }
}

pub struct Application {}

impl Application {
    #[tracing::instrument(skip(self), err(Debug))]
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("starting");
        Ok(())
    }
}
