use tokio_util::sync::CancellationToken;

mod bluetooth;
mod otel;

pub trait Configurable: Sized {
    fn from_env() -> anyhow::Result<Self>;
}

pub struct ApplicationConfig {
    otel: crate::otel::OtelConfig,
    bluetooth: crate::bluetooth::BluetoothConfig,
}

impl Configurable for ApplicationConfig {
    fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            otel: crate::otel::OtelConfig::from_env()?,
            bluetooth: crate::bluetooth::BluetoothConfig::from_env()?,
        })
    }
}

impl ApplicationConfig {
    pub async fn build(&self) -> anyhow::Result<Application> {
        self.otel.install()?;

        let cancel_token = CancellationToken::new();

        Ok(Application {
            bluetooth: self.bluetooth.build(cancel_token.child_token()).await?,
            cancel_token,
        })
    }
}

pub struct Application {
    bluetooth: crate::bluetooth::BluetoothCollector,
    cancel_token: CancellationToken,
}

impl Application {
    #[tracing::instrument(skip(self), err(Debug))]
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("starting");
        tokio::spawn(shutdown_signal(self.cancel_token));
        self.bluetooth.run().await?;
        Ok(())
    }
}

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown requested");

    token.cancel();
}
